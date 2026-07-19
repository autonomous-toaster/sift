## Context

`agent_mode()` calls `std::process::exit()` immediately after `dispatch_full()`. The recording in `dispatch()` uses `handle.spawn(fut)` which is fire-and-forget — the spawned task never executes before the process dies.

## Goals / Non-Goals

**Goals:**
- Ensure SQLite write completes before process exits
- No fragile timing hacks (sleeps, retries)

**Non-Goals:**
- Changing the recording mechanism (spawn vs block_on)
- Performance optimization

## Decisions

1. **Let main return normally** — `std::process::exit()` bypasses Rust's drop and tokio's shutdown. By returning from `main()`, the tokio runtime drains pending tasks naturally. This is the most reliable approach (council consensus).

2. **agent_mode returns exit code** — Change from `fn agent_mode(...) -> Result<()>` to `fn agent_mode(...) -> Result<i32>`. The caller in `main()` calls `std::process::exit()` after the async context completes.

## Risks / Trade-offs

1. **None** — This is a straightforward refactor with no behavioral change other than fixing the race condition.