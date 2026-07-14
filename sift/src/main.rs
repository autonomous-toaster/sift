//! sift — AI-optimized shell proxy.
//!
//! A PTY-based bash proxy with Lua plugin system for command interception
//! and output optimization. Reduces LLM token consumption by caching,
//! filtering, and transforming command output.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use sift_core::lua::{SiftContext, SiftLua};
use sift_core::session::Session;

/// AI-optimized shell proxy — Lua-plugin-based command interception.
#[derive(Parser)]
#[command(name = "sift", version, about)]
struct Args {
    /// Execute a command string and exit (agent mode).
    #[arg(short = 'c')]
    command: Option<String>,

    /// Start an interactive REPL session.
    #[arg(long = "shell")]
    shell: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut session = Session::from_env();
    session.open_store().await;

    let ctx = SiftContext {
        cwd: session.cwd.clone(),
        cmd_count: 0,
        env: session.env.clone(),
        session_id: session.session_id.clone(),
        raw_bytes: 0,
        filtered_bytes: 0,
    };

    let store = session.store.map(std::sync::Arc::new);
    let mut lua = SiftLua::new(store, ctx)?;

    // Load built-in plugins
    load_builtin_plugins(&mut lua)?;
    // Load user plugins from filesystem
    load_user_plugins(&mut lua);

    match args.command {
        Some(cmd) => agent_mode(&lua, &cmd),
        None if args.shell => repl_mode(&lua),
        None => {
            // No command and no --shell: read from stdin
            repl_mode(&lua)
        }
    }
}

/// Load all built-in Lua plugins.
fn load_builtin_plugins(lua: &mut SiftLua) -> Result<()> {
    lua.load_plugin_from_str("bash", include_str!("../plugins/bash.lua"))?;
    lua.load_plugin_from_str("cat", include_str!("../plugins/cat.lua"))?;
    lua.load_plugin_from_str("command", include_str!("../plugins/command.lua"))?;
    lua.load_plugin_from_str("git_status", include_str!("../plugins/git_status.lua"))?;
    lua.load_plugin_from_str("reset", include_str!("../plugins/reset.lua"))?;
    Ok(())
}

/// Load user plugins from `~/.config/sift/plugins/*.lua` and `SIFT_PLUGINS`.
fn load_user_plugins(lua: &mut SiftLua) {
    // Scan ~/.config/sift/plugins/
    if let Some(home) = dirs::home_dir() {
        let user_dir = home.join(".config").join("sift").join("plugins");
        if user_dir.exists() {
            load_plugins_from_dir(lua, &user_dir);
        }
    }
    // Scan SIFT_PLUGINS env var
    if let Ok(extra) = std::env::var("SIFT_PLUGINS") {
        for path in extra.split(':') {
            let dir = PathBuf::from(path);
            if dir.is_dir() {
                load_plugins_from_dir(lua, &dir);
            }
        }
    }
}

/// Load all `.lua` files from a directory as plugins.
fn load_plugins_from_dir(lua: &mut SiftLua, dir: &PathBuf) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "lua") {
                if let Ok(code) = std::fs::read_to_string(&path) {
                    let name = path.file_stem().unwrap_or_default().to_string_lossy();
                    if let Err(e) = lua.load_plugin_from_str(&name, &code) {
                        eprintln!("sift: failed to load plugin {}: {e}", path.display());
                    }
                }
            }
        }
    }
}

/// Agent mode: execute a command and output the result.
fn agent_mode(lua: &SiftLua, cmd: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (name, args) = if parts.is_empty() {
        return Ok(());
    } else {
        (parts[0], parts[1..].iter().map(ToString::to_string).collect::<Vec<_>>())
    };

    let (output, exit_code, _plugin) = lua.dispatch(name, &args, None)?;

    if !output.is_empty() {
        io::stdout().write_all(output.as_bytes())?;
        io::stdout().flush()?;
    }

    std::process::exit(exit_code);
}

/// REPL mode: read commands from stdin.
fn repl_mode(lua: &SiftLua) -> Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("sift$ ");
        io::stdout().flush()?;

        input.clear();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let cmd = input.trim();
        if cmd.is_empty() {
            continue;
        }
        if cmd == "exit" {
            break;
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let (name, args) = (parts[0], parts[1..].iter().map(ToString::to_string).collect::<Vec<_>>());

        let (output, exit_code, _plugin) = lua.dispatch(name, &args, None)?;

        if !output.is_empty() {
            io::stdout().write_all(output.as_bytes())?;
        }
        if exit_code != 0 {
            eprintln!("exit code: {exit_code}");
        }
    }

    Ok(())
}
