# Autoresearch: Optimize sift for fast execution, zero-copy, high throughput

## Objective
Optimize the sift shell proxy for maximum execution speed, minimal allocations (zero-copy where possible), and high throughput command processing — while keeping the Lua plugin API friendly and ergonomic.

The hot paths are:
1. **`exec_command`** — spawns bash, reads stdout/stderr via threads with 4KB chunks, `String::from_utf8_lossy`, lots of allocations
2. **`dispatch`** — plugin dispatch with Lua interop, context table creation, args table creation
3. **`try_pipeline`** — pipeline optimization with bash subprocess + plugin dispatch
4. **Cache operations** — file-based cache with JSON metadata reads/writes
5. **`StdinReader`** — streaming stdin reader with `Arc<Mutex<...>>` interior mutability

## Metrics
- **Primary**: test_µs (µs, lower is better) — median execution time of `cargo test --workspace`
- **Secondary**: build_µs (µs) — compile time of workspace

## How to Run
`./.auto/measure.sh` — outputs `METRIC name=number` lines.

## Files in Scope
- `sift-core/src/lua/exec.rs` — command execution (hot path #1)
- `sift-core/src/lua/api.rs` — dispatch, pipeline, plugin matching (hot path #2, #3)
- `sift-core/src/lua/api_reg_cache.rs` — cache operations (hot path #4)
- `sift-core/src/lua/api_reg_io.rs` — I/O API registration
- `sift-core/src/lua/api_reg_args.rs` — args parsing
- `sift-core/src/lua/api_reg_ext.rs` — extension API
- `sift-core/src/lua/api.rs` — main API
- `sift-core/src/lua/stdin_reader.rs` — stdin reader (hot path #5)
- `sift-core/src/lua/mod.rs` — SiftLua struct, plugin loading
- `sift-core/src/session.rs` — session store (SQLite)
- `sift-core/src/classifier.rs` — command classification
- `sift/src/main.rs` — binary entry point

## Off Limits
- Do NOT change Lua plugin files (`.lua` files in `plugins/` and `sift/plugins/`)
- Do NOT add new dependencies without clear justification
- Do NOT remove existing API functions from the Lua `sift.*` table
- Do NOT change the `Cargo.toml` workspace lints (unsafe_code = forbid, unwrap_used = deny, etc.)

## Constraints
- All existing tests must pass
- `unsafe_code` is forbidden — no unsafe blocks
- No `unwrap()`, `expect()`, or `panic!()` — use proper error handling
- Clippy must pass: `cargo clippy --workspace --all-targets -- -Dwarnings`
- Keep the Lua plugin API ergonomic — don't break existing plugin patterns
- No new dependencies unless they provide significant performance gains

## What's Been Tried
*(Update this section as experiments accumulate.)*

### Initial baseline
- Current test suite: ~260ms (105 tests)
- Key bottleneck: `exec_command` uses 4KB read chunks, threads for stdout/stderr, `String::from_utf8_lossy` per chunk, `Arc<Mutex<String>>` for collecting output
- `dispatch` creates new Lua tables for ctx and args on every call
- Pipeline optimization runs bash subprocess then dispatches to plugin

### Run 1-3: exec_command BufReader 64KB (+10.2%)
- Changed from 4KB raw read() to BufReader with 64KB buffer
- Avoided redundant String::clone in no-transform path
- Wrote bytes directly to stdout instead of print! macro
- **Result**: 859,008µs (best so far at that point)

### Run 4: HashMap plugin lookup + non-blocking record_conversation (+7.9%)
- Added pattern_map: HashMap<String, usize> for O(1) plugin matching
- Changed record_conversation to always spawn thread instead of block_in_place
- **Result**: 881,106µs (slightly worse than best, within noise)

### Run 5: session_id_str cache (DISCARDED)
- Added session_id_str field to SiftLua
- **Result**: 978,664µs — worse than baseline, reverted

### Run 6: cwd_str cache + args_table as_str (+9.0%)
- Added cwd_str: String to SiftContext to avoid repeated to_string_lossy()
- Changed args_table.set(i+1, arg.clone()) to args_table.set(i+1, arg.as_str())
- **Result**: 870,411µs

### Run 7: session_id_str cache (retry) (+6.6%)
- Added session_id_str to SiftLua, used in dispatch and record_conversation
- **Result**: 893,357µs

### Run 8: Re-measure (no code changes) (+8.5%)
- **Result**: 875,146µs — stable improvement

### Run 9: Pre-created ctx table template (+10.6%)
- Created ctx table template with static fields (cwd, session_id) in SiftLua::new
- Retrieve from registry and update only cmd_count/command/merge_stderr on dispatch
- **Result**: 855,218µs (new best)

### Run 10: parse_fd_redirects fast-path (+10.1%)
- Added fast-path check to avoid Vec allocation when no fd redirect patterns present
- **Result**: 859,753µs

### Run 11: Nudge text push_str instead of format!() (+12.0%)
- Used String::push_str instead of format!() for nudge text concatenation
- **Result**: 841,890µs (new best)

### Run 12: plugin_name as_str instead of cloned() (+11.8%)
- Changed record_conversation to accept Option<&str> instead of Option<String>
- **Result**: 843,921µs

### Run 13: raw_get instead of get for result table (+11.5%)
- Used raw_get instead of get for result table lookups (status, output, exit_code, streamed, raw_bytes)
- **Result**: 846,200µs

### Run 14: Confirmation run (+11.1%)
- **Result**: 849,808µs — stable at ~11% improvement
- **Confidence**: 6.6× noise floor

### Bottleneck Analysis
- SiftLua::new() avg: **95µs** (63 instances = ~6ms, only 2.3% of test time)
- dispatch avg: **2.1µs** (<0.2% of test time)
- **Conclusion**: Further micro-optimizations have diminishing returns. The ~11% gain is real but further improvements require architectural changes (reuse Lua VM, thread pool) that are out of scope for this session.
