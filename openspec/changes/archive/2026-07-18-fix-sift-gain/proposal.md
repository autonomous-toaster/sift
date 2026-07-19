## Why

`sift gain` was implemented as a Lua plugin (`gain.lua`) dispatched via `sift -c 'sift gain'`. This is wrong — it should be a CLI flag (`sift --gain`). Additionally, the `record_conversation` function uses `Handle::current().block_on()` which panics when called from within a tokio runtime. Clippy denies `unwrap_used`, `expect_used`, and `panic` were added to Cargo.toml but not all violations were fixed. A Justfile recipe to verify these lint rules are present is missing.

## What Changes

- Move `sift gain` from Lua plugin to CLI flag `--gain` in `sift/src/main.rs`
- Remove `plugins/gain.lua` (replaced by CLI flag)
- Fix `record_conversation` to not panic when no tokio runtime is available
- Fix all remaining clippy violations (`unwrap_used`, `expect_used`, `panic`)
- Add Justfile recipe `check-lint-rules` that parses Cargo.toml and verifies `unwrap_used`, `expect_used`, `panic` are present in workspace lints

## Capabilities

### New Capabilities
- (none)

### Modified Capabilities
- `sift-api`: `sift.gain.report()` remains as a Rust-registered Lua function (for programmatic access), but the primary entry point is now `sift --gain`

## Impact

- **sift/src/main.rs**: Add `--gain` CLI flag, handle it by querying session store and printing report
- **plugins/gain.lua**: Delete (replaced by CLI flag)
- **sift-core/src/lua/api.rs**: Fix `record_conversation` to handle non-tokio contexts without panic
- **sift-core/src/lua/api_reg_io.rs**: Fix remaining clippy violations
- **sift-core/src/lua/api_reg_cache.rs**: Fix `expect()` violation
- **Justfile**: Add `check-lint-rules` recipe
- **Cargo.toml**: Already has `unwrap_used`, `expect_used`, `panic` denies (added previously)