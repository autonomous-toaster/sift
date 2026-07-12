#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

//! baish — AI-optimized shell.
//!
//! A minimal POSIX shell that replaces bash for AI coding agents.
//! Intercepts commands, dispatches to optimized plugins, and caches
//! results to reduce token consumption.
//!
//! Usage:
//!   baish              Interactive shell (reads from stdin)
//!   baish -c <cmd>     Execute a command string and exit
//!   baish -l           Login shell (sources profile files)

mod builtins;
mod dispatcher;
mod parser;
mod plugin;
mod plugins;
mod session;

use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::Result;
use clap::Parser;

use crate::dispatcher::dispatch;
use crate::plugin::PluginRegistry;
use crate::plugins::cat::CatPlugin;
use crate::session::Session;

/// AI-optimized shell — minimal POSIX shell with plugin-based command optimization.
#[derive(Parser)]
#[command(name = "baish", version, about)]
struct Args {
    /// Execute a command string and exit (non-interactive).
    #[arg(short = 'c')]
    command: Option<String>,

    /// Act as a login shell (sources /etc/profile and ~/.profile).
    #[arg(short = 'l')]
    login: bool,
}

/// Source a file line by line through baish's dispatch.
async fn source_file(session: &mut Session, registry: &PluginRegistry, path: &Path, label: &str) {
    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Err(e) = dispatch(session, registry, line).await {
                        eprintln!("baish: {label} error: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("baish: could not read {label}: {e}");
            }
        }
    }
}

/// Source .bashrc on startup.
async fn source_bashrc(session: &mut Session, registry: &PluginRegistry) {
    let home = std::env::var("HOME").ok();
    if let Some(home) = home {
        let bashrc = Path::new(&home).join(".bashrc");
        source_file(session, registry, &bashrc, ".bashrc").await;
    }
}

/// Source login profile files.
async fn source_profile(session: &mut Session, registry: &PluginRegistry) {
    // /etc/profile (system-wide)
    source_file(session, registry, Path::new("/etc/profile"), "/etc/profile").await;

    // ~/.profile (user-specific)
    let home = std::env::var("HOME").ok();
    if let Some(home) = home {
        let profile = Path::new(&home).join(".profile");
        source_file(session, registry, &profile, ".profile").await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut session = Session::from_env();
    session.open_store().await;
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(CatPlugin));

    // Source profile files for login shells
    if args.login {
        source_profile(&mut session, &registry).await;
    }

    // Source .bashrc for interactive shells (or always, like bash does)
    source_bashrc(&mut session, &registry).await;

    // Non-interactive mode: execute -c command and exit
    if let Some(cmd) = args.command {
        match dispatch(&mut session, &registry, &cmd).await {
            Ok(output) => {
                io::stdout().write_all(&output)?;
            }
            Err(e) => {
                eprintln!("baish: error: {e}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // Interactive mode: REPL loop
    {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin.lock());
        let mut line = String::new();
        let mut stdout = io::stdout();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    eprintln!("baish: read error: {e}");
                    break;
                }
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed == "exit" || trimmed.starts_with("exit ") {
                let code = trimmed
                    .strip_prefix("exit ")
                    .and_then(|s| s.trim().parse::<i32>().ok())
                    .unwrap_or(0);
                std::process::exit(code);
            }

            match dispatch(&mut session, &registry, trimmed).await {
                Ok(output) => {
                    if let Err(e) = stdout.write_all(&output) {
                        eprintln!("baish: write error: {e}");
                        break;
                    }
                    if let Err(e) = stdout.flush() {
                        eprintln!("baish: flush error: {e}");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("baish: error: {e}");
                }
            }

            session.cmd_count += 1;
        }
        drop(reader);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baish_c_flag() {
        // Test that -c executes a command and exits
        let args = Args::try_parse_from(["baish", "-c", "echo hello"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, Some("echo hello".to_string()));
        assert!(!args.login);
    }

    #[test]
    fn test_baish_l_flag() {
        let args = Args::try_parse_from(["baish", "-l"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.command.is_none());
        assert!(args.login);
    }

    #[test]
    fn test_baish_lc_flags() {
        let args = Args::try_parse_from(["baish", "-l", "-c", "echo test"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.command, Some("echo test".to_string()));
        assert!(args.login);
    }

    #[test]
    fn test_baish_no_flags() {
        let args = Args::try_parse_from(["baish"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.command.is_none());
        assert!(!args.login);
    }
}
