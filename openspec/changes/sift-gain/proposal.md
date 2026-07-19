## Why

`sift` reduces token consumption by intercepting commands and transforming output. But there's no way to measure how much it saves. Without data, optimization is guesswork. The `conversation_cache` table already exists in SQLite — it tracks `raw_bytes`, `filtered_bytes`, `reduction_bps`, `plugin_name`, and `output_format` — but nothing populates it in the real dispatch flow. We need to wire it up and expose a `sift gain` command that reports token reduction, plugin-level breakdowns, and bypass counts.

## What Changes

- Wire `record_conversation()` into the real dispatch flow in `api.rs` — extract `raw_bytes` from plugin results, compute `filtered_bytes`, persist to SQLite
- Track bypasses (via `command.lua` plugin) as passthrough entries with 0% reduction
- Add `sift.gain.report()` Rust API function accessible from Lua
- Create `gain.lua` plugin dispatching `sift gain` with flags: `--verbose`, `--json`, `--all`, `--session <id>`
- Update `cat.lua` and `sift-read.lua` to report `raw_bytes` in their return tables

## Capabilities

### New Capabilities
- `gain`: Aggregated token-reduction reporting from the SQLite session store, with per-plugin and per-session breakdowns

### Modified Capabilities
- `sift-api`: Add `sift.gain.report()` as a new Rust-registered function
- `plugin-tests`: Add tests for the gain reporting path

## Impact

- **sift-core/src/lua/api.rs**: Wire recording after plugin execution and passthrough
- **sift-core/src/lua/api_reg_io.rs**: Add `register_gain()` and `generate_gain_report()`
- **sift-core/src/lua/mod.rs**: Wire `register_gain()` into `register_sift_table()`
- **plugins/cat.lua**: Add `raw_bytes` to return table
- **plugins/sift-read.lua**: Add `raw_bytes` to handled and unchanged return paths
- **plugins/gain.lua**: New plugin — dispatches `sift gain` with flags
- **sift-core/src/lua/tests_plugins.rs**: Test gain report query path
- No schema changes to `session.rs` — `conversation_cache` table already exists