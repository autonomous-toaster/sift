## Why

The sift codebase has accumulated dead code, misleading code paths, and a correctness bug that affect maintainability and reliability. A thorough code review identified several issues that should be fixed before they cause real problems.

### Problems

1. **Stale `ctx.cwd` after `cd`** — When a command like `cd /tmp && cat foo` is dispatched, the Lua context table still reports the old working directory. Any plugin that reads `ctx.cwd` to resolve relative paths gets wrong results. This is a correctness bug.

2. **Dead code accumulation** — Several code paths and data structures are never used in production:
   - `SessionStore::upsert_file_cache()` and the `file_cache` SQLite table are never called from production code (only from tests)
   - The `Err(_)` branch in `record_conversation()` creates a new tokio runtime as fallback, but `dispatch()` is always called from `#[tokio::main]`, making this branch unreachable
   - `#![allow(dead_code)]` at the crate level hides these from the compiler

3. **`sift.exit()` uses hard `process::exit()`** — Plugins can call `sift.exit(code)` to signal exit codes, but the implementation calls `std::process::exit()` which skips destructors, buffer flushes, and cleanup. The exit code should be stored and returned through the normal dispatch path.

4. **No binary integration tests** — The main entry point (`agent_mode()`, `repl_mode()`), plugin loading logic, and CLI argument parsing have zero test coverage.

5. **Lua runtime has unrestricted stdlib** — Plugins have access to the full Lua standard library including `debug`, `io`, `os`, and `load`, which is unnecessary for a plugin system and allows accidental or malicious misuse.

### Non-Goals

- Full plugin sandboxing or capability system — this is a personal tool, not a multi-tenant platform
- Architectural rewrites (no library swaps, no new dependencies)
- Performance optimizations beyond removing dead code paths
- Adding new features or plugin APIs

## What

This change fixes the identified issues through targeted deletions and small refactors:

- Fix `ctx.cwd` staleness by updating the Lua context template after directory changes
- Remove dead code: `file_cache` table, `upsert_file_cache()`, dead `Err(_)` branch, `#![allow(dead_code)]` lint
- Reimplement `sift.exit()` as a stored exit code returned through `dispatch_full()`
- Add integration tests for the binary entry point
- Restrict Lua stdlib to `BASIC | MATH | STRING | TABLE`

## Impact

- ~120 lines of dead code removed
- One correctness bug fixed
- One safety issue fixed (hard `process::exit()`)
- Basic Lua sandboxing added
- Test coverage extended to the binary entry point
- No behavior changes for any shipped plugin
