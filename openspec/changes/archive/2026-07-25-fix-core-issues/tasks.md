## 1. Fix stale `ctx.cwd` after `cd`

- [x] 1.1 Change `peel_cd_prefix()` to return `Option<(String, String)>` — the new cwd and the rest of the command
- [x] 1.2 In `dispatch_full()`, update the Lua ctx template's `cwd` field after a successful `cd`
- [x] 1.3 Verify that plugins reading `ctx.cwd` after `cd /tmp && ...` get the correct directory

## 2. Remove dead code

- [x] 2.1 Remove `FileCacheEntry` struct, `get_file_cache()`, `upsert_file_cache()` from `session.rs`
- [x] 2.2 Remove `CREATE TABLE IF NOT EXISTS file_cache` from `SessionStore::open()`
- [x] 2.3 Remove `DELETE FROM file_cache` from `clear_session()`
- [x] 2.4 Remove dead `Err(_)` branch in `record_conversation()` — replace with `expect()` since handle is guaranteed
- [x] 2.5 Remove `#![allow(dead_code)]` from `lib.rs` and `main.rs`
- [x] 2.6 Fix any remaining dead code exposed by removing the lint (delete or add targeted `#[allow(...)]`)
- [x] 2.7 Remove associated tests for removed `file_cache` functionality
- [x] 2.8 Run `cargo test` to verify no regressions

## 3. Reimplement `sift.exit()` as stored exit code

- [x] 3.1 Add `exit_code: Cell<Option<i32>>` field to `SiftLua` struct
- [x] 3.2 Change `sift.exit()` Lua function to store the code instead of calling `process::exit()`
- [x] 3.3 In `dispatch_full()`, return the stored exit code
- [x] 3.4 In `agent_mode()` in `main.rs`, call `process::exit()` with the returned code
- [x] 3.5 Run `cargo test` to verify no regressions

## 4. Add integration tests

- [x] 4.1 Add `assert_cmd` and `predicates` dev-dependencies to `sift/Cargo.toml`
- [x] 4.2 Create `sift/tests/cli.rs` with tests for agent mode (`-c`), exit code propagation, `--gain`, and `--shell`
- [x] 4.3 Run integration tests to verify they pass

## 5. Restrict Lua stdlib

- [x] 5.1 Change `Lua::new()` to `Lua::new_with(StdLib::MATH | StdLib::STRING | StdLib::TABLE)` in `SiftLua::new()`
- [x] 5.2 Run `cargo test` to verify all shipped plugins still work
