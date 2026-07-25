# CLI Subcommands — Design

## CLI Shape

```
sift                    → REPL (default)
sift -c "cmd"           → agent mode (one-shot)
sift --shell            → REPL (explicit, same as default)
sift gain               → gain report
sift gain --daily       → with daily time-series
sift gain --weekly      → with weekly time-series
sift gain --verbose     → per-command breakdown + sequential dups
sift gain --reset       → clear current session's gain data
sift gain --reset --all → clear ALL gain data
sift gain --json        → JSON output
sift gain --all         → show all sessions
sift gain --session "id" → filter by session
sift gain --since <ts>  → filter by timestamp
```

## Arg Parsing (clap)

```rust
#[derive(clap::Parser)]
#[command(name = "sift")]
struct Cli {
    /// Execute a command string and exit (agent mode)
    #[arg(short = 'c')]
    exec: Option<String>,

    /// Subcommands
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(clap::Subcommand)]
enum CliCommand {
    /// Show gain report (token reduction stats)
    Gain {
        #[arg(long)]
        daily: bool,
        #[arg(long)]
        weekly: bool,
        #[arg(long, short)]
        verbose: bool,
        #[arg(long)]
        reset: bool,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        since: Option<i64>,
    },
}
```

When `command` is `None` and `exec` is `None` → enter REPL mode.

## Main Dispatch

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut session = Session::from_env();
    session.open_store().await;

    match (&cli.command, &cli.exec) {
        (Some(CliCommand::Gain { .. }), _) => {
            handle_gain(&session, cli.command.unwrap()).await?;
        }
        (None, Some(cmd)) => {
            handle_agent(&session, cmd).await?;
        }
        (None, None) => {
            handle_repl(&session).await?;
        }
    }
    Ok(())
}
```

## Gain Handler

```rust
async fn handle_gain(session: &Session, args: &GainArgs) -> Result<()> {
    if args.reset {
        if args.all {
            session.store.reset_all_gain_data().await?;
        } else {
            session.store.reset_session_gain_data(&session.session_id).await?;
        }
        println!("Gain data cleared.");
        return Ok(());
    }

    let flags = GainFlags {
        verbose: args.verbose,
        json: args.json,
        all: args.all,
        session: args.session.clone(),
        since: args.since,
        daily: args.daily,
        weekly: args.weekly,
    };
    let report = generate_gain_report(&store, effective_session, &flags).await?;
    let output = format_gain_report(&report, effective_session);
    print!("{output}");
}
```

## Session Store — Reset Methods

```rust
/// Clear gain data for the current session only.
pub async fn reset_session_gain_data(&self, session_id: &str) -> Result<()> {
    let prefix = format!("{session_id}_");
    sqlx::query(
        "DELETE FROM conversation_cache WHERE item_type = 'command_output' AND item_id LIKE ?1"
    )
    .bind(format!("{prefix}%"))
    .execute(&self.pool)
    .await?;
    Ok(())
}

/// Clear ALL gain data across all sessions.
pub async fn reset_all_gain_data(&self) -> Result<()> {
    sqlx::query("DELETE FROM conversation_cache WHERE item_type = 'command_output'")
        .execute(&self.pool)
        .await?;
    Ok(())
}
```

## Full Command Storage

Currently `dispatch()` stores `cmd` (just the first token, e.g. `"cat"`). Change to store the reconstructed full command (name + args).

In `dispatch()`, reconstruct the full command before recording:

```rust
let full_cmd = if args.is_empty() {
    cmd.to_string()
} else {
    let quoted: Vec<String> = args.iter().map(|a| sh_quote(a)).collect();
    format!("{} {}", cmd, quoted.join(" "))
};
```

Then pass `full_cmd` instead of `cmd.to_string()` to `record_conversation()`.

This means `SKIP=cargo-clippy git commit -m "fix"` is stored as `"SKIP=cargo-clippy git commit -m fix"` instead of just `"SKIP=cargo-clippy"`.

## Files Changed

| File | Change |
|------|--------|
| `sift/src/main.rs` | Restructure `Args` → `Cli` with subcommands, add `handle_gain()`, default REPL |
| `sift-core/src/session.rs` | Add `reset_session_gain_data()`, `reset_all_gain_data()` |
| `sift-core/src/lua/api.rs` | Store full reconstructed command instead of first token |
| `sift/tests/cli.rs` | Update tests for new CLI shape |
