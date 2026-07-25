# CLI Subcommands

## Purpose

Restructure sift's CLI from flat flags to subcommands, make REPL the default mode, and add gain data reset.

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Restructure `Args` to `Cli` with `CliCommand` enum using clap subcommands |
| T1.2 | Wire main dispatch: subcommand → handler, no subcommand + no `-c` → REPL |
| T2.1 | Add `reset_session_gain_data()` and `reset_all_gain_data()` to `SessionStore` |
| T2.2 | Add `handle_gain()` with `--reset` and `--reset --all` logic |
| T3.1 | Reconstruct full command (name + args) in `dispatch()` before recording |
| T4.1 | Update integration tests for new CLI shape |
| T4.2 | Run `cargo test` to verify no regressions |

## Requirements

### Requirement: CLI is restructured to subcommands

T1.1 SHALL complete BEFORE T1.2 SHALL run.

#### Scenario: Subcommand structure

- **WHEN** T1.1 runs
- **THEN** `sift gain` SHALL be a subcommand with `--daily`, `--weekly`, `--verbose`, `--reset`, `--all`, `--json`, `--session`, `--since` flags

- **WHEN** T1.2 runs
- **THEN** `sift` with no arguments SHALL start the REPL

- **WHEN** T1.2 runs
- **THEN** `sift -c "cmd"` SHALL execute the command and exit

- **WHEN** T1.2 runs
- **THEN** `sift --shell` SHALL start the REPL

### Requirement: Gain data can be reset

T2.1 SHALL complete BEFORE T2.2 SHALL run.

#### Scenario: Reset gain data

- **WHEN** T2.1 runs
- **THEN** `reset_session_gain_data()` SHALL delete `command_output` entries matching the session prefix

- **WHEN** T2.1 runs
- **THEN** `reset_all_gain_data()` SHALL delete all `command_output` entries

- **WHEN** T2.2 runs
- **THEN** `sift gain --reset` SHALL clear the current session's gain data

- **WHEN** T2.2 runs
- **THEN** `sift gain --reset --all` SHALL clear all gain data

### Requirement: Full command is stored

T3.1 SHALL complete BEFORE T4.1 SHALL run.

#### Scenario: Full command recording

- **WHEN** T3.1 runs
- **THEN** `dispatch()` SHALL store the reconstructed full command (name + args) instead of just the first token

- **WHEN** T3.1 runs
- **THEN** `SKIP=cargo-clippy git commit -m "fix"` SHALL be stored as `"SKIP=cargo-clippy git commit -m fix"` not `"SKIP=cargo-clippy"`

### Requirement: Tests pass

T4.1 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: Tests

- **WHEN** T4.1 runs
- **THEN** integration tests SHALL cover the new CLI shape

- **WHEN** T4.2 runs
- **THEN** `cargo test` SHALL pass with no regressions
