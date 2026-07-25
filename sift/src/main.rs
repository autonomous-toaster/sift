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

use sift_core::lua::{cleanup_cache, SiftContext, SiftLua};
use sift_core::session::Session;

/// AI-optimized shell proxy — Lua-plugin-based command interception.
#[derive(Parser)]
#[command(name = "sift", version, about)]
struct Cli {
    /// Execute a command string and exit (agent mode).
    #[arg(short = 'c')]
    exec: Option<String>,

    /// Start an interactive REPL session (default when no args).
    #[arg(long)]
    shell: bool,

    /// Subcommands.
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(clap::Subcommand)]
/// Available subcommands.
enum CliCommand {
    /// Show gain report (token reduction stats).
    Gain {
        /// Show daily time-series.
        #[arg(long)]
        daily: bool,

        /// Show weekly time-series.
        #[arg(long)]
        weekly: bool,

        /// Show verbose per-command breakdown.
        #[arg(long, short)]
        verbose: bool,

        /// Reset gain tracking data.
        #[arg(long)]
        reset: bool,

        /// Apply reset to all sessions (not just current).
        #[arg(long)]
        all: bool,

        /// Output as JSON.
        #[arg(long)]
        json: bool,

        /// Filter by specific session ID.
        #[arg(long)]
        session: Option<String>,

        /// Filter by timestamp (unix ms).
        #[arg(long)]
        since: Option<i64>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut session = Session::from_env();
    session.open_store().await;

    // Handle subcommands
    match &cli.command {
        Some(CliCommand::Gain {
            daily,
            weekly,
            verbose,
            reset,
            all,
            json,
            session: session_filter,
            since,
        }) => {
            if let Some(ref store) = session.store {
                if *reset {
                    if *all {
                        store.reset_all_gain_data().await?;
                    } else if let Some(ref sid) = session.session_id {
                        store.reset_session_gain_data(sid).await?;
                    }
                    println!("Gain data cleared.");
                    return Ok(());
                }
                let flags = sift_core::lua::api_reg_io::GainFlags {
                    verbose: *verbose,
                    json: *json,
                    all: *all,
                    session: session_filter.clone(),
                    since: *since,
                    daily: *daily,
                    weekly: *weekly,
                };
                let effective_session = session.session_id.as_deref();
                let report = sift_core::lua::api_reg_io::generate_gain_report(
                    store,
                    effective_session,
                    &flags,
                )
                .await?;
                let output =
                    sift_core::lua::api_reg_io::format_gain_report(&report, effective_session);
                print!("{output}");
            } else {
                eprintln!("sift: no session store. Set AI_SESSION to enable tracking.");
            }
            return Ok(());
        }
        None => {}
    }

    let ctx = SiftContext {
        cwd: session.cwd.clone(),
        cwd_str: session.cwd.display().to_string(),
        cmd_count: std::cell::Cell::new(0),
        env: session.env.clone(),
        session_id: session.session_id.clone(),
        raw_bytes: 0,
        filtered_bytes: 0,
    };

    let store = session.store.map(std::sync::Arc::new);

    // Clean up expired cache entries at startup
    if let Some(ref sid) = session.session_id {
        cleanup_cache(sid, 86_400_000); // 24h default TTL
    }

    let mut lua = SiftLua::new(store, ctx)?;

    // Load built-in plugins
    load_builtin_plugins(&mut lua)?;
    // Load user plugins from filesystem
    load_user_plugins(&mut lua);

    let exit_code = match cli.exec {
        Some(cmd) => agent_mode(&lua, &cmd)?,
        None => {
            repl_mode(&lua)?;
            0
        }
    };

    std::process::exit(exit_code);
}

/// Load all built-in Lua plugins.
fn load_builtin_plugins(lua: &mut SiftLua) -> Result<()> {
    lua.load_plugin_from_str("bash", include_str!("../plugins/bash.lua"))?;
    lua.load_plugin_from_str("command", include_str!("../plugins/command.lua"))?;
    lua.load_plugin_from_str("reset", include_str!("../plugins/reset.lua"))?;
    Ok(())
}

/// Load user plugins from `plugins/`, `~/.config/sift/plugins/*.lua` and `SIFT_PLUGINS`.
fn load_user_plugins(lua: &mut SiftLua) {
    // Scan top-level plugins/ directory (shipped optional plugins)
    let project_plugins = std::path::PathBuf::from("plugins");
    if project_plugins.is_dir() {
        load_plugins_from_dir(lua, &project_plugins);
    }
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
fn agent_mode(lua: &SiftLua, cmd: &str) -> Result<i32> {
    let (_output, exit_code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>)?;

    Ok(exit_code)
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

        let (output, exit_code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>)?;

        if !output.is_empty() {
            io::stdout().write_all(output.as_bytes())?;
        }
        if exit_code != 0 {
            eprintln!("exit code: {exit_code}");
        }
    }

    Ok(())
}
