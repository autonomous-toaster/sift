## Context

The current baish codebase has a Rust-based plugin system (`Plugin` trait, `PluginRegistry`) with a single hardcoded `CatPlugin`. The binary is named `baish` and includes a ratatui TUI. The workspace has three crates: `baish-core`, `baish-filters`, `baish`. The session store uses SQLite at `~/.baish/sessions.db`. Output storage writes to `/tmp/baish/`.

The new design pivots from "bash replacement" to "shell proxy" — a tool named `sift` that intercepts commands, runs them through Lua plugins, and optimizes output for LLM consumption.

## Goals / Non-Goals

**Goals:**
- Replace Rust `Plugin` trait with a Lua plugin runtime using mlua
- Define a comprehensive `sift.*` Lua API covering execution, caching, hashing, filesystem, encoding, and querying
- Ship default plugins (bash.lua, cat.lua, command.lua, reset.lua, git_status.lua) embedded in the binary
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

### D3 — `sift.exec()` uses std::process::Command with pipes (not PTY)

`sift.exec(cmd) → stdout, stderr, exit_code` uses `std::process::Command` with piped stdout and stderr. No PTY is involved. This provides clean stdout/stderr separation and eliminates pager blocking. Every process receives `PAGER=cat`, `TERM=dumb`, `EDITOR=true` in its environment to ensure non-interactive, plain-text output. See ADR-0005 and STD-005.

**State variable**: `exec_mechanism ∈ {std_process_command}`

### D4 — Automatic raw output storage with format detection

Every call to `sift.exec()` saves the raw output to a temp file at `/tmp/sift/<session>/<cmd_count>_<slug>.log`. The format is detected from content (JSON if starts with `{` or `[`, TOON if header matches, otherwise text). The path is recorded in `sift.meta` and used for bypass notices when plugins transform or truncate output.

**State variable**: `output_storage ∈ {auto_save, format_detected}`

### D5 — Bypass via `command` builtin

The `command` bash builtin is repurposed as the bypass mechanism. A built-in `command.lua` plugin (priority 1000) matches the `command` prefix and returns `passthrough`, which tells sift to execute the real binary directly without plugin interception. No new syntax — `command cat foo` already means "bypass shell-defined behavior" in bash.

**State variable**: `bypass_mode ∈ {passthrough, plugin}`

### D6 — Token tracking: automatic + plugin-overridable

`sift.meta.filtered_bytes` is computed automatically from the `output` field in the plugin's return value. `sift.meta.raw_bytes` defaults to `filtered_bytes` but plugins can override it (e.g., when they transform JSON to TOON, the raw JSON is larger). `sift.meta.stdout` and `sift.meta.stderr` are set automatically after each `sift.exec()` call. Metrics are stored in the session DB per command.

**State variable**: `token_tracking ∈ {auto, plugin_override}`

### D7 — `sift.fs.*` mirrors pi tool signatures

`sift.fs.read(path, {offset?, limit?})` matches pi's `read` tool. `sift.fs.edit(path, edits)` matches pi's `edit` tool. This enables future agent integration where sift plugins can perform file operations using the same interface the agent uses.

**State variable**: `fs_api ∈ {pi_compatible}`

### D8 — TOON and jq as optional transformations

`sift.toon.encode/decode` and `sift.jq.query` are available to plugins but not applied automatically. Plugins opt in by calling these functions. The `toon-format` and `jaq` crates are required dependencies (not feature-gated).

**State variable**: `encoding ∈ {json, toon}`

### D9 — Cache: separate sift_cache table, ctx-first interface

The sift cache uses a dedicated `sift_cache` table (key, session_id, created_at) instead of the `conversation_cache` table. The cache key is purely content-based (`path:hash`). Session scoping is handled by the cache layer via `ctx.session_id`, not by encoding session_id into the key. The interface is `sift.cache.has(ctx, key)`, `sift.cache.set(ctx, key)`, `sift.cache.reset(ctx)`. See ADR-0006, STD-006, STD-007.

**State variable**: `cache_storage ∈ {sift_cache_table}`

### D10 — Stderr: separate capture via pipes, three-value return

`sift.exec()` returns `(stdout, stderr, exit_code)` instead of `(output, exit_code)`. stdout and stderr are captured separately via `std::process::Command` pipes. Plugins that don't need stderr use `_` to ignore it: `local out, _, code = sift.exec(cmd)`. See ADR-0007, STD-005.

**State variable**: `stderr_capture ∈ {separate_pipes}`

### D11 — Reset: built-in plugin + Lua API

`sift.cache.reset(ctx)` clears all cache entries for the current session. A built-in `reset.lua` plugin (pattern: `reset`, priority: 1000) calls this API, making cache reset callable from the shell. The `command` builtin provides an escape hatch: `command reset` runs the real bash reset. See ADR-0008.

**State variable**: `reset_mechanism ∈ {plugin_and_api}`

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
│  │  │bash.lua  │ │cat.lua │ │command.lua│ │reset.lua │ │  │
│  │  │(-1000)   │ │(0)     │ │(1000)     │ │(1000)    │ │  │
│  │  └──────────┘ └────────┘ └───────────┘ └──────────┘ │  │
│  │  ┌──────────┐                                        │  │
│  │  │git_status│  User plugins from filesystem           │  │
│  │  │.lua (0)  │  → override built-ins by priority       │  │
│  │  └──────────┘                                        │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Core (Rust)                                          │  │
│  │  ┌──────────────────┐  ┌────────────────────────┐    │  │
│  │  │ Process Execution│  │ Parser                │    │  │
│  │  │(std::process::   │  │(brush-parser)         │    │  │
│  │  │ Command + pipes) │  └────────────────────────┘    │  │
│  │  └──────────────────┘                                │  │
│  │  ┌──────────────────┐  ┌────────────────────────┐    │  │
│  │  │ Session Store    │  │ sift_cache Table       │    │  │
│  │  │(SQLite, ~/.sift/)│  │(per-session scoped)    │    │  │
│  │  └──────────────────┘  └────────────────────────┘    │  │
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
       │       └── returns passthrough → run /bin/cat via std::process::Command
       │
       ├── reset.lua (priority 1000) → "reset"
       │       └── calls sift.cache.reset(ctx) → clears session cache
       │       └── returns "[sift] ok"
       │
       ├── cat.lua (priority 0) → "cat foo.rs"
       │       └── reads file via sift.fs.read()
       │       └── checks sift.cache.has(ctx, path:hash)
       │       └── returns handled/unchanged
       │
       └── bash.lua (priority -1000) → default fallback
               └── calls sift.exec(ctx.command) → bash via std::process::Command
               └── returns raw output as-is
       │
       ▼
  sift processes result:
    - "handled" → emit output, compute filtered_bytes
    - "unchanged" → emit message + bypass notice
    - "truncated" → emit summary + full_output_path notice
    - "passthrough" → run real binary via std::process::Command, emit raw output
       │
       ▼
  Store metrics in session DB (raw_bytes, filtered_bytes, plugin_name)
```

## Risks / Trade-offs

- **[Lua parsing overhead]** → Plugin loading happens once at startup. Per-command dispatch is a Lua function call — negligible cost compared to process I/O.
- **[TOON dependency weight]** → `toon-format` adds ~10 dependencies. Acceptable for the token savings it provides.
- **[jaq complexity]** → Full jq syntax is powerful but complex. Plugins that use it will be more complex. Trade-off accepted — "full jq or no jq."
- **[CWD-relative sandboxing]** → `sift.fs.*` restricts reads to CWD by default. This is a soft boundary — no hard sandboxing. Users who need wider access can use `sift.exec("cat /etc/passwd")` which goes through bash.
- **[Breaking rename]** → Existing `baish` users need to update their PATH and config. Mitigation: provide a `baish → sift` symlink for transition.
- **[No PTY for interactive commands]** → Commands that require a TTY (top, less, vim) will not work through sift.exec(). Acceptable — AI agents do not run interactive commands. The environment contract (PAGER=cat, TERM=dumb, EDITOR=true) ensures non-interactive behavior.
- **[EDITOR=true may surprise]** → Setting EDITOR=true prevents git from opening an editor for rebase/commit. This is intentional — AI agents should not be blocked by editor prompts. If an editor is genuinely needed, the agent can set EDITOR=vim explicitly before the command.

## Open Questions

- Plugin hot-reload: should `SIGHUP` trigger plugin reload? Deferred.
- `sift --tui` mode: what session data should it display? Deferred to future change.
- Lua module system: should plugins be able to `require` other Lua modules? Deferred.
- `portable-pty` removal: should the dependency be removed from Cargo.toml, or kept for future TUI mode? Deferred — remove if no code references it.

## References

- [ADR-0005: std::process::Command for sift.exec()](../../decisions/adrs/0005-std-process-command-for-exec.md)
- [ADR-0006: Separate sift_cache table](../../decisions/adrs/0006-separate-sift-cache-table.md)
- [ADR-0007: Stderr capture via pipes](../../decisions/adrs/0007-stderr-capture-pipes.md)
- [ADR-0008: Reset plugin](../../decisions/adrs/0008-reset-plugin.md)
- [STD-005: sift.exec() API and environment contract](../../decisions/standards/STD-005-exec-api.md)
- [STD-006: sift_cache table schema](../../decisions/standards/STD-006-sift-cache-schema.md)
- [STD-007: Plugin cache interface](../../decisions/standards/STD-007-cache-interface.md)
