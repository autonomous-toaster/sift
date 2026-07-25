## 1. Restructure CLI

- [x] 1.1 Restructure `Args` to `Cli` with `CliCommand` enum using clap subcommands
- [x] 1.2 Wire main dispatch: subcommand → handler, no subcommand + no `-c` → REPL

## 2. Add gain reset

- [x] 2.1 Add `reset_session_gain_data()` and `reset_all_gain_data()` to `SessionStore`
- [x] 2.2 Add `handle_gain()` with `--reset` and `--reset --all` logic

## 3. Store full command

- [x] 3.1 Reconstruct full command (name + args) in `dispatch()` before recording

## 4. Verify

- [x] 4.1 Update integration tests for new CLI shape
- [x] 4.2 Run `cargo test` to verify no regressions
