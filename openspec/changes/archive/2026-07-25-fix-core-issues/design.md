## Context

The sift codebase is a Rust workspace with two crates: `sift-core` (Lua runtime, session store, classifier) and `sift` (binary entry point, plugin loading). The Lua dispatch path is the central hot path — every command flows through `SiftLua::dispatch()`.

The issues to fix are scattered across the codebase but are all small, targeted changes. No architectural changes are needed.

## Goals / Non-Goals

**Goals:**
- Fix stale `ctx.cwd` after `cd` commands
- Remove dead code: `file_cache` table, `upsert_file_cache()`, dead `Err(_)` branch, `#![allow(dead_code)]` lint
- Reimplement `sift.exit()` as stored exit code
- Add integration tests for the binary entry point
- Restrict Lua stdlib to prevent misuse

**Non-Goals:**
- No library swaps (keep sqlx, keep brush-parser)
- No new dependencies
- No API changes for plugins (except `sift.exit()` behavior is preserved)
- No performance optimization beyond removing dead code

## Detailed Design

### Fix 1: Stale `ctx.cwd` after `cd`

**Location**: `sift-core/src/lua/api.rs` — `peel_cd_prefix()` and `dispatch_full()`

**Current behavior**: `peel_cd_prefix()` calls `std::env::set_current_dir()` but never updates the Lua context table's `cwd` field. The ctx template is created once in `SiftLua::new()` and never updated.

**Fix**: Change `peel_cd_prefix()` to return the new working directory alongside the rest of the command. In `dispatch_full()`, after a successful `cd`, update the `ctx_template_key` registry value's `cwd` field.

```rust
// New signature
fn peel_cd_prefix(input: &str) -> Option<(String, String)> {
    // Returns (new_cwd, rest_of_command)
}

// In dispatch_full():
if let Some((new_cwd, rest)) = peel_cd_prefix(full_cmd) {
    if let Some(ref key) = self.ctx_template_key {
        if let Ok(t) = self.lua.registry_value::<mlua::Table>(key) {
            let _ = t.set("cwd", &new_cwd);
        }
    }
    return self.dispatch_full(&rest, stdin);
}
```

### Fix 2: Remove dead code

**2a. Remove `file_cache` table and `upsert_file_cache()`**

**Location**: `sift-core/src/session.rs`

Remove:
- `FileCacheEntry` struct
- `get_file_cache()` method
- `upsert_file_cache()` method
- `CREATE TABLE IF NOT EXISTS file_cache` from `open()`
- `DELETE FROM file_cache` from `clear_session()`
- Associated tests

**2b. Remove dead `Err(_)` branch in `record_conversation()`**

**Location**: `sift-core/src/lua/api.rs` — `record_conversation()`

Remove the `Err(_)` branch that creates a new tokio runtime. Replace with an `expect()` since the handle is guaranteed to exist:

```rust
let handle = tokio::runtime::Handle::try_current()
    .expect("record_conversation must be called from a tokio runtime");
tokio::task::block_in_place(move || {
    let _ = handle.block_on(record);
});
```

**2c. Remove `#![allow(dead_code)]`**

**Location**: `sift-core/src/lib.rs`, `sift/src/main.rs`

Remove the lint. Fix any resulting compiler errors by either deleting dead code or adding targeted `#[allow(...)]` annotations where the code is intentionally unused (e.g., for future use).

### Fix 3: Reimplement `sift.exit()` as stored exit code

**Location**: `sift-core/src/lua/api_reg_cache.rs` (registration), `sift-core/src/lua/api.rs` (dispatch), `sift-core/src/lua/mod.rs` (SiftLua struct)

**Current behavior**: `sift.exit(code)` calls `std::process::exit(code)` — a hard exit that skips destructors.

**Fix**: Add an `exit_code: Cell<Option<i32>>` field to `SiftLua`. When a plugin calls `sift.exit(code)`, store the code instead of calling `process::exit()`. In `dispatch_full()`, check the stored exit code and return it. The caller (`agent_mode()` in `main.rs`) calls `process::exit()` with the returned code.

```rust
// In SiftLua struct:
exit_code: std::cell::Cell<Option<i32>>,

// In register_exec():
let exit_fn = self.lua.create_function(move |_, (_ctx, code): (Table, i32)| {
    exit_code.set(Some(code));
    Ok(())
})?;

// In dispatch_full():
let exit_code = self.exit_code.get().unwrap_or(0);
// ... return exit_code from dispatch_full
```

### Fix 4: Add integration tests

**Location**: `sift/tests/` (new directory)

Add integration tests using `assert_cmd` and `predicates` crates:

```rust
// tests/cli.rs
use assert_cmd::Command;

#[test]
fn test_agent_mode_cat() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("echo hello");
    cmd.assert().success().stdout(predicate::str::contains("hello"));
}

#[test]
fn test_agent_mode_exit_code() {
    let mut cmd = Command::cargo_bin("sift").unwrap();
    cmd.arg("-c").arg("exit 42");
    cmd.assert().code(42);
}
```

### Fix 5: Restrict Lua stdlib

**Location**: `sift-core/src/lua/mod.rs` — `SiftLua::new()`

**Current**: `let lua = Lua::new();`

**Fix**: 
```rust
let lua = Lua::new_with(mlua::StdLib::BASIC | mlua::StdLib::MATH | mlua::StdLib::STRING | mlua::StdLib::TABLE);
```

This removes `io.*`, `os.*`, `debug.*`, and `load*` from the Lua runtime. All shipped plugins only use `string`, `math`, `table`, and basic functions (all in the allowed set).

## Dependencies

- Fix 1 (ctx.cwd) is independent
- Fix 2 (dead code) is independent
- Fix 3 (sift.exit) is independent
- Fix 4 (integration tests) depends on Fix 2 (removing dead code reduces noise)
- Fix 5 (stdlib restriction) is independent

All fixes can be implemented in any order.

## Risks and Mitigations

- **Fix 3 (sift.exit)**: Existing plugins that call `sift.exit()` will no longer cause an immediate process exit. The exit code is returned at the end of dispatch. This is a behavior change — if a plugin calls `sift.exit(1)` and then continues executing, the exit code will be 1 but the plugin's output will still be emitted. Mitigation: document that `sift.exit()` sets the exit code for the current command, it does not immediately terminate.
- **Fix 5 (stdlib restriction)**: If a user plugin uses `io.*` or `os.*` from Lua, it will break. Mitigation: this is a personal tool; the user controls all plugins. The restriction only affects Lua-level APIs, not `sift.*` APIs.
