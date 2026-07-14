## Why

The current baish implementation uses a Rust Plugin trait with hardcoded plugins (CatPlugin). This requires recompilation to add or modify behavior, limits extensibility, and prevents users from customizing how commands are intercepted and optimized. A Lua plugin system enables dynamic, user-extensible command interception with a rich runtime API for caching, transformation, and token optimization.

## What Changes

- **BREAKING**: Replace Rust `Plugin` trait with Lua plugin runtime (mlua) — all plugins are Lua scripts
- **BREAKING**: `sift.exec()` uses `std::process::Command` with pipes (not PTY) — returns `(stdout, stderr, exit_code)`
- **BREAKING**: Cache uses dedicated `sift_cache` table with per-session scoping via `ctx` — key is content-based (`path:hash`), not session-encoded
- **NEW**: `sift.*` Lua API with namespaces for process execution, caching, hashing, filesystem, JSON/TOON encoding, jq queries, environment access, command classification, and token tracking
- **NEW**: Default built-in plugins (bash.lua, cat.lua, command.lua, reset.lua, git_status.lua) embedded in the binary
- **NEW**: User plugin discovery at `~/.config/sift/plugins/*.lua` and `SIFT_PLUGINS` env var
- **NEW**: Automatic raw output storage at `/tmp/sift/<session>/<cmd_count>_<slug>.log` with format detection
- **NEW**: Bypass notices — when a plugin returns "unchanged" or truncated output, sift appends a notice telling the LLM how to get full content via the `command` builtin
- **NEW**: Token reduction tracking — per-command metrics (raw_bytes, filtered_bytes, reduction_pct) stored in session DB
- **NEW**: `sift.fs.read/write/edit` API mirroring pi tool signatures for future agent integration
- **NEW**: TOON format support via `toon-format` crate for token-optimized output encoding
- **NEW**: jq query support via `jaq` crate for JSON data filtering in plugins
- **NEW**: `sift.cache.has/set/reset` with ctx-first interface — per-session cache scoping
- **NEW**: `reset.lua` built-in plugin — clears cache for current session, callable as `sift -c "reset"`
- **NEW**: Environment contract — `PAGER=cat`, `TERM=dumb`, `EDITOR=true` for all subprocesses
- **REMOVED**: Rust `Plugin` trait, `PluginRegistry`, `cat_plugin.rs`
- **REMOVED**: TUI from main binary (deferred to `sift --tui` mode)
- **REMOVED**: PTY-based command execution (portable-pty deferred to TUI mode)
- **MODIFIED**: `baish-core` → `sift-core`, `baish-filters` → merged into sift-core, `baish` → `sift`

## Capabilities

### New Capabilities
- `lua-runtime`: mlua-based plugin execution with sandboxed environment and `sift.*` API
- `plugin-system`: Plugin discovery, priority-based resolution, longest-prefix matching, built-in + user plugins
- `sift-api`: The `sift.*` Lua API surface (exec, cache, hash, fs, json, toon, jq, env, classify, meta, log, exit)
- `output-storage`: Automatic raw output saving to temp files with format detection and cleanup
- `token-tracking`: Per-command token reduction metrics stored in session database
- `bypass-mechanism`: The `command` builtin as a plugin that bypasses all other plugins
- `toon-encoding`: TOON format support for token-optimized structured data
- `jq-queries`: Full jq filter support for JSON data transformation in plugins
- `cache-system`: Dedicated `sift_cache` table with per-session scoping via ctx-first interface
- `reset-plugin`: Built-in `reset.lua` plugin for per-session cache clearing
- `environment-contract`: PAGER=cat, TERM=dumb, EDITOR=true for all subprocesses

### Modified Capabilities
- `session-store`: Add `sift_cache` table, remove cache entries from `conversation_cache`
- `exec-api`: Switch from PTY to std::process::Command with pipes, return (stdout, stderr, exit_code)

## Impact

- **Binary name**: `baish` → `sift` (installed as `sift`, not `bash`)
- **Config directory**: `~/.baish/` → `~/.sift/`
- **Temp directory**: `/tmp/baish/` → `/tmp/sift/`
- **Dependencies added**: `mlua`, `toon-format`, `jaq`, `serde`, `serde_json`
- **Dependencies removed**: `ratatui`, `crossterm` (deferred to TUI mode)
- **Dependencies kept**: `portable-pty`, `nix`, `tokio`, `sqlx`, `brush-parser`, `clap`, `chrono`, `sha2`, `hex`
- **Workspace**: `baish-core` → `sift-core`, `baish-filters` merged into sift-core, `baish` → `sift`
