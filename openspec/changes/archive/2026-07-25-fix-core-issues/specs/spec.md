## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Change `peel_cd_prefix()` to return `Option<(String, String)>` |
| T1.2 | In `dispatch_full()`, update the Lua ctx template's `cwd` field after a successful `cd` |
| T1.3 | Verify that plugins reading `ctx.cwd` after `cd /tmp && ...` get the correct directory |
| T2.1 | Remove `FileCacheEntry` struct, `get_file_cache()`, `upsert_file_cache()` from `session.rs` |
| T2.2 | Remove `CREATE TABLE IF NOT EXISTS file_cache` from `SessionStore::open()` |
| T2.3 | Remove `DELETE FROM file_cache` from `clear_session()` |
| T2.4 | Remove dead `Err(_)` branch in `record_conversation()` |
| T2.5 | Remove `#![allow(dead_code)]` from `lib.rs` and `main.rs` |
| T2.6 | Fix any remaining dead code exposed by removing the lint |
| T2.7 | Remove associated tests for removed `file_cache` functionality |
| T2.8 | Run `cargo test` to verify no regressions |
| T3.1 | Add `exit_code: Cell<Option<i32>>` field to `SiftLua` struct |
| T3.2 | Change `sift.exit()` Lua function to store the code instead of calling `process::exit()` |
| T3.3 | In `dispatch_full()`, return the stored exit code |
| T3.4 | In `agent_mode()` in `main.rs`, call `process::exit()` with the returned code |
| T3.5 | Run `cargo test` to verify no regressions |
| T4.1 | Add `assert_cmd` and `predicates` dev-dependencies to `sift/Cargo.toml` |
| T4.2 | Create `sift/tests/cli.rs` with integration tests |
| T4.3 | Run integration tests to verify they pass |
| T5.1 | Change `Lua::new()` to `Lua::new_with(StdLib::BASIC \| StdLib::MATH \| StdLib::STRING \| StdLib::TABLE)` |
| T5.2 | Run `cargo test` to verify all shipped plugins still work |

### Requirement: ctx.cwd is updated after cd

T1.2 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: cd changes directory and updates context

**WHEN** T1.1 runs and returns a new working directory
**THEN** T1.2 SHALL update the Lua context template's `cwd` field

**WHEN** T1.2 completes
**THEN** T1.3 SHALL verify that `ctx.cwd` matches `std::env::current_dir()`

### Requirement: file_cache removal is sequential

T2.1 SHALL complete BEFORE T2.2 SHALL run.
T2.2 SHALL complete BEFORE T2.3 SHALL run.

#### Scenario: file_cache table is removed

**WHEN** T2.1 removes the `FileCacheEntry` struct and methods
**THEN** T2.2 SHALL remove the `CREATE TABLE IF NOT EXISTS file_cache` statement

**WHEN** T2.2 completes
**THEN** T2.3 SHALL remove the `DELETE FROM file_cache` from `clear_session()`

### Requirement: Err branch removal is concurrent with file_cache cleanup

T2.4 SHALL complete CONCURRENTLY with T2.1.

#### Scenario: Err branch is removed concurrently

**WHEN** T2.1 removes the `FileCacheEntry` struct and methods
**THEN** T2.4 SHALL remove the dead `Err(_)` branch in `record_conversation()`

### Requirement: dead code lint is removed after cleanup

T2.5 SHALL complete AFTER T2.1, T2.2, T2.3, and T2.4 SHALL complete.

#### Scenario: dead_code lint is removed

**WHEN** T2.1, T2.2, T2.3, and T2.4 have removed all known dead code
**THEN** T2.5 SHALL remove `#![allow(dead_code)]` from `lib.rs` and `main.rs`

**WHEN** T2.5 completes
**THEN** T2.6 SHALL fix any remaining dead code exposed by the compiler

### Requirement: tests pass after dead code removal

T2.8 SHALL complete AFTER T2.7 SHALL complete.

#### Scenario: tests pass after cleanup

**WHEN** T2.7 removes tests for removed `file_cache` functionality
**THEN** T2.8 SHALL run `cargo test` and all tests SHALL pass

### Requirement: sift.exit() does not call process.exit()

T3.1 SHALL complete BEFORE T3.2 SHALL run.
T3.2 SHALL complete BEFORE T3.3 SHALL run.
T3.3 SHALL complete BEFORE T3.4 SHALL run.

#### Scenario: exit code is stored and returned

**WHEN** T3.1 adds the `exit_code` field to `SiftLua`
**THEN** T3.2 SHALL change `sift.exit()` to store the code instead of calling `process::exit()`

**WHEN** T3.2 completes
**THEN** T3.3 SHALL return the stored exit code from `dispatch_full()`

**WHEN** T3.3 completes
**THEN** T3.4 SHALL call `process::exit()` with the returned code in `agent_mode()`

### Requirement: sift.exit() tests pass

T3.5 SHALL complete AFTER T3.4 SHALL complete.

#### Scenario: tests pass after exit code refactor

**WHEN** T3.4 completes
**THEN** T3.5 SHALL run `cargo test` and all tests SHALL pass

### Requirement: integration tests are added

T4.1 SHALL complete BEFORE T4.2 SHALL run.
T4.2 SHALL complete BEFORE T4.3 SHALL run.

#### Scenario: integration tests are created

**WHEN** T4.1 adds `assert_cmd` and `predicates` dev-dependencies
**THEN** T4.2 SHALL create `sift/tests/cli.rs` with tests for agent mode, exit code, `--gain`, and `--shell`

**WHEN** T4.2 completes
**THEN** T4.3 SHALL run the integration tests and they SHALL pass

### Requirement: lua stdlib is restricted

T5.1 SHALL complete BEFORE T5.2 SHALL run.

#### Scenario: Lua stdlib is restricted

**WHEN** T5.1 changes `Lua::new()` to `Lua::new_with(StdLib::BASIC | StdLib::MATH | StdLib::STRING | StdLib::TABLE)`
**THEN** T5.2 SHALL run `cargo test` and all shipped plugins SHALL continue to work
