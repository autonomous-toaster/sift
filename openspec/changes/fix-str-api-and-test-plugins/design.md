## Context

`sift.str.*` functions were registered with `ctx: Table` as first parameter matching the `sift.fs.*` convention, but all plugins call them as pure functions. The existing 68 Rust tests never caught this because they only test inline Lua plugins, never loading the actual `.lua` files from disk.

## Goals / Non-Goals

**Goals:**
- Fix the `sift.str.*` signature mismatch so plugins work
- Add tests that load actual `.lua` plugin files from disk
- Fix all existing `just ci` violations (clippy, line limits, file size limits)

**Non-Goals:**
- Integration tests running the `sift` binary (deferred)
- Refactoring `session.rs` or `register_json_toon` line limits (out of scope, pre-existing)
- Changing plugin `.lua` files (they already call correctly)

## Decisions

1. **Remove `ctx` from `sift.str.*` signatures** — These are pure utility functions (string splitting, path matching). No plugin ever passes `ctx`. The `sift.fs.*` convention (all take `ctx`) is for I/O operations that need context.

2. **New `tests_plugins.rs` file** — Loading actual `.lua` files from disk is the only way to catch API contract mismatches. The file stays under 500 lines. Tests are deterministic: fixture files created in temp dirs, no external state.

3. **Smoke test loading ALL plugins** — Verifies every `sift.*` sub-table and function is visible from each plugin. Catches any future registration gaps.

## Risks / Trade-offs

1. **File path resolution** — Tests use `plugins/` relative path. Cargo test runs from workspace root (`baish/`), so `plugins/sift-read.lua` resolves correctly. Test will fail if run from a different cwd. → Mitigation: use `env!("CARGO_MANIFEST_DIR")` to resolve relative to `sift-core/` crate root.

2. **Line limit on `tests.rs`** — Already at 665 lines (limit 550). Moving plugin tests to `tests_plugins.rs` reduces pressure. → Already planned.