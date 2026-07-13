## Context

The current baish codebase has a Rust-based plugin system (`Plugin` trait, `PluginRegistry`) with a single hardcoded `CatPlugin`. The binary is named `baish` and includes a ratatui TUI. The workspace has three crates: `baish-core`, `baish-filters`, `baish`. The session store uses SQLite at `~/.baish/sessions.db`. Output storage writes to `/tmp/baish/`.

The new design pivots from "bash replacement" to "shell proxy" — a tool named `sift` that intercepts commands, runs them through Lua plugins, and optimizes output for LLM consumption.

## Goals / Non-Goals

**Goals:**
- Replace Rust `Plugin` trait with a Lua plugin runtime using mlua
- Define a comprehensive `sift.*` Lua API covering execution, caching, hashing, filesystem, encoding, and querying
- Ship default plugins (bash.lua, cat.lua, command.lua, cargo_test.lua, git_status.lua) embedded in the binary
- Support user plugins from `~/.config/sift/plugins/*.lua` and `SIFT_PLUGINS` env var
- Implement automatic raw output storage with format detection and bypass notices
- Track token reduction metrics per command in the session store
- Rename project from `baish` to `sift` with single binary and two modes (`--shell`, `--tui`)
- Support TOON format via `toon-format` crate for token-optimized output
- Support jq queries via `jaq` crate for JSON data filtering

**Non-Goals:**
- TUI mode — deferred to future change
- Windows support — Linux and macOS only
- Lua plugin sandboxing beyond CWD-relative path restrictions
- Plugin hot-reload — plugins are loaded at startup
- Subprocess plugins (Lua only, no external process plugins)

## Decisions

### D1 — Lua runtime: mlua with Lua 5.4

mlua is the most maintained Rust Lua binding. Lua 5.4 is the default target. The `send` feature is enabled for thread safety. The `serialize` feature enables serde integration for JSON/TOON conversion.

**State variable**: `lua_engine ∈ {mlua_lua54}`

### D2 — Plugin dispatch: priority-based with longest-prefix matching

Same algorithm as the current Rust `PluginRegistry` but implemented in Rust calling Lua. Built-in plugins have priority -1000. User plugins default to priority 0. The `command` plugin has priority 1000 to ensure it always matches first.

**State variable**: `plugin_resolution ∈ {longest_prefix, priority_tiebreak}`

### D3 — `sift.exec()` as the only PTY API (for now)

Council consensus: ship `sift.exec(cmd) → output, exit_code` as the only PTY function. The object API (`sift.pty.spawn()`) is deferred. `sift.exec()` always saves raw output to `/tmp/sift/<session>/<cmd_count>_<slug>.log` and records the path internally for bypass notices.

**State variable**: `pty_api ∈ {exec_only}`

### D4 — Automatic raw output storage with format detection

Every call to `sift.exec()` saves the raw PTY output to a temp file. The format is detected from content (JSON if starts with `{` or `[`, TOON if header matches, otherwise text). The path is recorded in `sift.meta` and used for bypass notices when plugins transform or truncate output.

**State variable**: `output_storage ∈ {auto_save, format_detected}`

### D5 — Bypass via `command` builtin

The `command` bash builtin is repurposed as the bypass mechanism. A built-in `command.lua` plugin (priority 1000) matches the `command` prefix and returns `passthrough`, which tells sift to execute the real binary directly without plugin interception. No new syntax — `command cat foo` already means "bypass shell-defined behavior" in bash.

**State variable**: `bypass_mode ∈ {passthrough, plugin}`

### D6 — Token tracking: automatic + plugin-overridable

`sift.meta.filtered_bytes` is computed automatically from the `output` field in the plugin's return value. `sift.meta.raw_bytes` defaults to `filtered_bytes` but plugins can override it (e.g., when they transform JSON to TOON, the raw JSON is larger). Metrics are stored in the session DB per command.

**State variable**: `token_tracking ∈ {auto, plugin_override}`

### D7 — `sift.fs.*` mirrors pi tool signatures

`sift.fs.read(path, {offset?, limit?})` matches pi's `read` tool. `sift.fs.edit(path, edits)` matches pi's `edit` tool. This enables future agent integration where sift plugins can perform file operations using the same interface the agent uses.

**State variable**: `fs_api ∈ {pi_compatible}`

### D8 — TOON and jq as optional transformations

`sift.toon.encode/decode` and `sift.jq.query` are available to plugins but not applied automatically. Plugins opt in by calling these functions. The `toon-format` and `jaq` crates are required dependencies (not feature-gated).

**State variable**: `encoding ∈ {json, toon}`

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  sift binary                                              │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Lua Runtime (mlua)                                  │  │
│  │                                                       │  │
│  │  ┌────────────────────┐  ┌────────────────────────┐  │  │
│  │  │  sift.* API        │  │  Plugin Registry       │  │  │
│  │  │  (Rust callbacks)  │  │  (priority dispatch)   │  │  │
│  │  └────────────────────┘  └────────────────────────┘  │  │
│  │                                                       │  │
│  │  Built-in plugins (embedded Lua strings):             │  │
│  │  ┌──────────┐ ┌────────┐ ┌───────────┐ ┌──────────┐ │  │
│  │  │bash.lua  │ │cat.lua │ │command.lua│ │cargo_test│ │  │
│  │  │(-1000)   │ │(0)     │ │(1000)     │ │.lua (0)  │ │  │
│  │  └──────────┘ └────────┘ └───────────┘ └──────────┘ │  │
│  │  ┌──────────┐                                        │  │
│  │  │git_status│  User plugins from filesystem           │  │
│  │  │.lua (0)  │  → override built-ins by priority       │  │
│  │  └──────────┘                                        │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Core (Rust)                                          │  │
│  │  ┌──────────┐  ┌──────────┐  ┌────────────────────┐  │  │
│  │  │ PTY      │  │ Parser   │  │ Session Store      │  │  │
│  │  │(portable │  │(brush)   │  │(SQLite, ~/.sift/)  │  │  │
│  │  │ -pty)    │  └──────────┘  └────────────────────┘  │  │
│  │  └──────────┘                                         │  │
│  │  ┌──────────────────┐  ┌────────────────────────┐    │  │
│  │  │ Output Storage   │  │ Token Tracking         │    │  │
│  │  │(/tmp/sift/...)   │  │(per-command metrics)    │    │  │
│  │  └──────────────────┘  └────────────────────────┘    │  │
│  └──────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Plugin dispatch flow

```
User runs: cat foo.rs
       │
       ▼
  Parse & classify
       │
       ▼
  Find matching plugins (longest-prefix, highest priority)
       │
       ├── command.lua (priority 1000) → "command cat foo"
       │       └── returns passthrough → run /bin/cat directly
       │
       ├── cat.lua (priority 0) → "cat foo.rs"
       │       └── calls sift.exec("cat foo.rs") → bash via PTY
       │       └── checks cache, returns handled/unchanged
       │
       └── bash.lua (priority -1000) → default fallback
               └── calls sift.exec(...) → bash via PTY
               └── returns raw output as-is
       │
       ▼
  sift processes result:
    - "handled" → emit output, compute filtered_bytes
    - "unchanged" → emit message + bypass notice
    - "truncated" → emit summary + full_output_path notice
    - "passthrough" → run real binary, emit raw output
       │
       ▼
  Store metrics in session DB (raw_bytes, filtered_bytes, plugin_name)
```

## Risks / Trade-offs

- **[Lua parsing overhead]** → Plugin loading happens once at startup. Per-command dispatch is a Lua function call — negligible cost compared to PTY I/O.
- **[TOON dependency weight]** → `toon-format` adds ~10 dependencies. Acceptable for the token savings it provides.
- **[jaq complexity]** → Full jq syntax is powerful but complex. Plugins that use it will be more complex. Trade-off accepted — "full jq or no jq."
- **[CWD-relative sandboxing]** → `sift.fs.*` restricts reads to CWD by default. This is a soft boundary — no hard sandboxing. Users who need wider access can use `sift.exec("cat /etc/passwd")` which goes through bash.
- **[Breaking rename]** → Existing `baish` users need to update their PATH and config. Mitigation: provide a `baish → sift` symlink for transition.

## Open Questions

- Plugin hot-reload: should `SIGHUP` trigger plugin reload? Deferred.
- `sift --tui` mode: what session data should it display? Deferred to future change.
- Lua module system: should plugins be able to `require` other Lua modules? Deferred.
