## Why

`sift --gain` shows 0 commands even after running commands. The recording in `dispatch()` spawns an async SQLite write via `handle.spawn(fut)`, but `agent_mode()` immediately calls `std::process::exit()` — killing the process before the spawned task runs. The write never happens.

## What Changes

- Change `agent_mode()` to return exit code instead of calling `std::process::exit()`
- In `main()`, call `std::process::exit()` after the async function returns, allowing the tokio runtime to drain pending tasks
- This ensures the SQLite write completes before the process exits

## Capabilities

### New Capabilities
- (none)

### Modified Capabilities
- (none — no spec-level behavior change, just a bug fix)

## Impact

- **sift/src/main.rs**: Remove `std::process::exit()` from `agent_mode()`, return exit code instead. Call `std::process::exit()` in `main()` after async context drains.