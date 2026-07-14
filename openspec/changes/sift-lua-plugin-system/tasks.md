## 1. Project rename and workspace restructure

- [x] 1.1 Rename workspace members: `baish-core` → `sift-core`, `baish-filters` → merge into sift-core, `baish` → `sift`. Update all Cargo.toml files and cross-references.
- [x] 1.2 Rename binary output to `sift` with `[[bin]] name = "sift"` in Cargo.toml. Remove ratatui and crossterm dependencies.
- [x] 1.3 Update config paths: `~/.baish/` → `~/.sift/`, `/tmp/baish/` → `/tmp/sift/`. Update `SessionStore::open()` and `OutputStore` paths.
- [x] 1.4 Remove Rust `Plugin` trait, `PluginRegistry`, `cat_plugin.rs`. Remove `baish-filters` crate (merge StreamFilter into sift-core if needed).

## 2. Lua runtime and plugin system

- [x] 2.1 Add `mlua` dependency with `lua54`, `send`, `serialize` features. Initialize Lua VM at startup in `sift-core`.
- [x] 2.2 Implement plugin registry in Rust: priority-based resolution with longest-prefix matching. Store plugin as Lua reference (not Rust trait object).
- [x] 2.3 Implement plugin loading: load Lua chunk, call it to get plugin table, validate required fields (name, execute), register in registry.
- [x] 2.4 Implement built-in plugin embedding: store default plugins (bash.lua, cat.lua, command.lua, cargo_test.lua, git_status.lua) as `&str` constants in Rust source.
- [x] 2.5 Implement user plugin discovery: scan `~/.config/sift/plugins/*.lua` and `SIFT_PLUGINS` env var paths. Load after built-ins.
- [x] 2.6 Implement plugin dispatch: find best matching plugin, call `plugin.execute(ctx, args, stdin)`, handle return values (handled, passthrough, unchanged, truncated, error).

## 3. sift.* API — Core functions

- [x] 3.1 Implement `sift.exec(cmd)`: spawn bash via portable-pty PTY, write command, read output, save raw output to temp file, return (output, exit_code).
- [x] 3.2 Implement `sift.cache.{get,set,has}`: session-scoped key-value cache backed by the session store's conversation_cache table.
- [x] 3.3 Implement `sift.hash.{sha256,md5}`: hash input data and return hex-encoded string.
- [x] 3.4 Implement `sift.fs.read(path, {offset?, limit?})`: read file with optional line range. Mirror pi's read tool signature.
- [x] 3.5 Implement `sift.fs.write(path, content)`: write content to file.
- [x] 3.6 Implement `sift.fs.edit(path, edits)`: apply multiple disjoint text replacements. Mirror pi's edit tool signature.
- [x] 3.7 Implement `sift.fs.stat(path)`: return file metadata (size, mtime, is_dir, is_file).
- [x] 3.8 Implement `sift.fs.exists(path)`: check if path exists.
- [x] 3.9 Implement `sift.json.{encode,decode}`: convert between Lua tables and JSON strings via serde_json.
- [x] 3.10 Implement `sift.toon.{encode,decode}`: convert between Lua tables and TOON strings via toon-format + serde_json.
- [x] 3.11 Implement `sift.jq.query(data, filter)`: execute jq filter on JSON data using jaq crate. Accept data as JSON string or Lua table.
- [x] 3.12 Implement `sift.env.{get,set}`: read/write environment variables.
- [x] 3.13 Implement `sift.classify(cmd)`: parse command with brush-parser, return {kind, name, args, is_piped, is_compound}.
- [x] 3.14 Implement `sift.{log,exit,output}`: logging (info/warn/error/debug), process exit, and output emission.
- [x] 3.15 Implement `sift.meta`: read-only fields (session_id, cmd_count, cwd) + writable raw_bytes. Compute filtered_bytes automatically from returned output.
- [x] 3.16 Implement `sift.token_count(text)`: estimate token count as len/4.

## 4. Output storage and format detection

- [x] 4.1 Implement automatic raw output saving in `sift.exec()`: write to `/tmp/sift/<session>/<cmd_count>_<slug>.log`.
- [x] 4.2 Implement format detection: inspect first bytes for JSON (`{` or `[`), TOON header, or text fallback.
- [x] 4.3 Implement temp file cleanup: remove session directory on exit. Configurable max disk usage with oldest-first eviction.

## 5. Token tracking and bypass notices

- [x] 5.1 Add columns to session store: `raw_bytes`, `filtered_bytes`, `reduction_pct`, `plugin_name`, `output_format` to conversation_cache.
- [x] 5.2 Compute and store per-command metrics after plugin execution. Compute reduction_pct as `(raw - filtered) / raw * 100`.
- [x] 5.3 Generate bypass notices: for "unchanged" → append `[sift] Use 'command <cmd>' for full content`. For "truncated" → append `[sift] Full output: <path>` and `[sift] Use 'command cat <path>' for raw output`.

## 6. Default plugins

- [x] 6.1 Write `bash.lua`: default fallback plugin (priority -1000). Calls `sift.exec()` with the command, returns raw output.
- [x] 6.2 Write `cat.lua`: file read plugin (priority 0). Reads file via `sift.fs.read()`, caches by hash, returns "unchanged" on cache hit.
- [x] 6.3 Write `command.lua`: bypass plugin (priority 1000). Matches "command" prefix, returns passthrough.
- [x] 6.4 Write `cargo_test.lua`: test output plugin (priority 0). Parses `cargo test --message-format=json` output via `sift.jq.query()`, returns TOON-encoded summary.
- [x] 6.5 Write `git_status.lua`: git status plugin (priority 0). Fingerprints output, returns "working tree clean" on match.

## 7. Migration and cleanup

- [x] 7.1 Update Justfile: rename targets from baish to sift, update paths.
- [x] 7.2 Update docs/examples/cat.lua to use new `sift.*` API.
- [x] 7.3 Remove old `openspec/changes/baish-pty-architecture` (completed and superseded).
- [x] 7.4 Verify `just ci` passes with all changes.

## 8. Cache redesign and stderr handling

- [x] 8.1 Add `sift_cache` table to SessionStore with cache_has, cache_set, cache_reset methods. Remove cache entries from conversation_cache.
- [x] 8.2 Rewrite `sift.cache.*` Lua API: ctx-first interface (has/set/reset), no boolean value, session scoping via ctx.session_id.
- [x] 8.3 Rewrite `run_pty()` → `exec_command()` using std::process::Command with pipes. Return (stdout, stderr, exit_code). Set PAGER=cat, TERM=dumb, EDITOR=true.
- [x] 8.4 Update all plugins to use new sift.exec() return values: `local out, err, code = sift.exec(cmd)`.
- [x] 8.5 Update cat.lua: cache key = path:hash (no session_id), use sift.cache.has/set(ctx, key).
- [x] 8.6 Update git_status.lua: extend ctx.command + args instead of hardcoding, remove 2>&1, use new sift.exec() signature.
- [x] 8.7 Write reset.lua: built-in plugin (pattern: reset, priority: 1000), calls sift.cache.reset(ctx), returns "[sift] ok".
- [x] 8.8 Remove portable-pty dependency if no longer referenced. Remove pty.rs module.
- [x] 8.9 Update STD-003 (plugin architecture) to reflect new plugin interface and sift.exec() signature.
- [x] 8.10 Verify `just ci` passes with all changes.
