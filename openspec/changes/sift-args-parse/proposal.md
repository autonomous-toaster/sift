## Why

Every Lua plugin reimplements the same ad-hoc argument parsing loop — manual index tracking, flag scanning, value consumption, and error handling. This is ~40% of all plugin code (152 lines across 8 plugins), copy-pasted with slight variations, and every new flag or plugin requires re-inventing the same while-loop. The result is fragile, inconsistent, and a barrier to adding new plugins.

## What Changes

- **NEW** `sift.args.parse()` — a Rust-based declarative argument parser exposed as a Lua function
- **BREAKING** All 8 shipped plugins (`cat.lua`, `head.lua`, `tail.lua`, `sed.lua`, `sift-read.lua`, `git-commit.lua`, `curl.lua`, `openspec.lua`) converted to use `sift.args.parse()` instead of manual parsers
- **BREAKING** `rtk.lua` unchanged (no arg parsing needed)
- Old manual parsers removed from all plugins

## Capabilities

### New Capabilities

- `args-parsing`: Declarative argument parsing for Lua plugins — define flags, positional args, and options in a spec table; get back a parsed result table or nil+error

### Modified Capabilities

- None — this is purely an internal refactor of plugin argument handling. No spec-level behavior changes.

## Impact

- **New file**: `sift-core/src/lua/api_reg_args.rs` — Rust implementation of `sift.args.parse()`
- **Modified**: `sift-core/src/lua/mod.rs` — add module
- **Modified**: `sift-core/src/lua/api.rs` — wire `register_args()`
- **Modified**: All 8 plugins in `plugins/` — replace manual parsers with `sift.args.parse()` calls
- **No new dependencies** — pure Rust stdlib + mlua
