## Why

The `sift.str.*` API functions (`split_lines`, `slice_text`, `is_sensitive`) were registered with a `ctx: Table` first parameter following the `sift.fs.*` convention, but all plugins call them without `ctx` — they are pure utility functions. This causes a runtime crash: `bad argument #1: error converting Lua string to table`.

The bug was missed because the 68 existing Rust tests never load the actual `.lua` plugin files from disk — they use inline Lua plugins that don't exercise the real plugin code paths.

## What Changes

- Remove `ctx` parameter from `sift.str.split_lines()`, `sift.str.slice_text()`, and `sift.str.is_sensitive()` Rust function signatures
- Add `tests_plugins.rs` that loads actual `.lua` files from `plugins/` directory and verifies they work with the full `sift.*` API
- Update README API reference to note that `sift.str.*` functions are pure and don't take `ctx`

## Capabilities

### New Capabilities
- `plugin-tests`: Unit tests that load actual `.lua` plugin files from disk, verify `sift.*` API availability, and test plugin execution with fixture files

### Modified Capabilities
- `sift-api`: The `sift.str` sub-table signature changes — `split_lines`, `slice_text`, and `is_sensitive` no longer accept a `ctx` parameter

## Impact

- **sift-core/src/lua/api_reg_io.rs**: `register_str()` — remove `ctx` from 3 closures
- **sift-core/src/lua/tests/tests_plugins.rs**: New file (~250 lines) with smoke test + per-plugin tests
- **README.md**: Update API reference for `sift.str.*` to document pure-function signatures
- No changes to any `.lua` plugin files — they already call `sift.str.*` without `ctx` (correct call site, wrong registration)