## Context

`sift` has a SQLite-backed `conversation_cache` table with `raw_bytes`, `filtered_bytes`, `reduction_bps`, `plugin_name`, and `output_format` columns. The `record_conversation()` method on `SessionStore` exists and is tested in isolation, but nothing calls it from the real dispatch flow. The data is schema'd but never populated.

The dispatch flow in `api.rs` (`dispatch()` method) is the single point where plugin output is finalized. This is the natural recording hook.

## Goals / Non-Goals

**Goals:**
- Wire `record_conversation()` into `dispatch()` so every plugin execution is recorded
- Track `raw_bytes` per command — plugins report it via an optional return field
- Track bypasses (passthrough via `command.lua`) as 0% reduction entries
- Expose `sift.gain.report()` as a Rust-registered Lua function
- Create `gain.lua` plugin dispatching `sift gain` with `--verbose`, `--json`, `--all`, `--session <id>`
- Update `cat.lua` and `sift-read.lua` to report `raw_bytes`

**Non-Goals:**
- Real-time streaming gain tracking (only recorded post-dispatch)
- Schema changes to `conversation_cache` — the existing table is sufficient
- Changing `bash.lua`, `head.lua`, `tail.lua`, `sed.lua` — they report no `raw_bytes`, fallback to 0% reduction
- pi integration — that's a separate concern

## Decisions

1. **raw_bytes as optional plugin return field** — Plugins that know their raw size (file readers) report it. Others don't. `dispatch()` falls back to `filtered_bytes` when absent, meaning 0% reduction. This is sufficient and necessary — no overhead for simple plugins, accurate for the ones that matter.

2. **Recording dispatch as the hook** — `dispatch()` is the single choke point where output is finalized. Recording here catches every path: handled, unchanged, passthrough. No need to instrument each plugin.

3. **Tokio::spawn for recording** — The SQLite write is async and non-blocking. Spawning avoids blocking the Lua dispatch. The `store` is `Arc<SessionStore>` so it's Send + Sync.

4. **gain.lua plugin** — A standard Lua plugin matching `sift gain` pattern. It calls `sift.gain.report()` with flags forwarded as arguments. The Rust function queries SQLite and returns a formatted string.

5. **Bypass tracking via plugin_name** — Passthrough entries have `plugin_name = "command"` and `output_format = "passthrough"`. The gain report filters these out from reduction stats and counts them separately.

## Risks / Trade-offs

1. **Tokio::spawn errors silently ignored** — If the SQLite write fails, the error is dropped. → Mitigation: acceptable for a telemetry path. The command output is already returned to the caller.

2. **SQLite contention** — Multiple concurrent dispatches could queue on the single-connection pool. → Mitigation: pool is `max_connections(1)`, writes serialize naturally. Acceptable for a single-threaded CLI tool.

3. **Gain without AI_SESSION** — If no session_id is set, there's no session store and no recording. → Mitigation: `sift gain` shows a message guiding the user to set `AI_SESSION`.