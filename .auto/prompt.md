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
