## 1. Wire recording in dispatch

- [x] 1.1 Wire `record_conversation()` into `dispatch()` — extract optional `raw_bytes` from plugin result, compute `filtered_bytes` from final output length (including nudge text), call `record_conversation()` via `tokio::spawn`
- [x] 1.2 Capture raw/filtered bytes in passthrough path — set `plugin_name = "command"`, `output_format = "passthrough"`, `raw_bytes = filtered_bytes` (0% reduction)
- [x] 1.3 Add test for recording in `tests_plugins.rs` — verify `conversation_cache` is populated after a dispatch with known raw/filtered bytes

## 2. Update plugins to report raw_bytes

- [x] 2.1 Add `raw_bytes` to cat.lua return — report file size from `sift.fs.stat` result
- [x] 2.2 Add `raw_bytes` to sift-read.lua return — report file size in handled and unchanged paths

## 3. Add sift.gain.report() Rust function

- [x] 3.1 Implement `register_gain()` and `generate_gain_report()` in `api_reg_io.rs` — query `conversation_cache`, aggregate stats (total commands, raw bytes, filtered bytes, reduction %, bypass count, per-plugin breakdown), support `--verbose`, `--json`, `--all`, `--session`, `--since` flags
- [x] 3.2 Wire `register_gain()` into `register_sift_table()` in `mod.rs`
- [x] 3.3 Add test for gain report queries in `tests_plugins.rs` — insert test data, query with default/JSON flags, verify aggregate output

## 4. Create gain.lua plugin

- [x] 4.1 Create `plugins/gain.lua` — match `"sift gain"` pattern, forward flags (`--verbose`, `--json`, `--all`, `--session`, `--since`) as args to `sift.gain.report()`
- [x] 4.2 Add test for gain.lua dispatch in `tests_plugins.rs` — verify `sift gain` and `sift gain --json` produce expected output