## 1. CLI flag + remove plugin

- [x] 1.1 Add `--gain` flag to `Args` in `main.rs` — when set, open session store, query gain report, print, exit
- [x] 1.2 Remove `plugins/gain.lua`

## 2. Fix panics and clippy violations

- [x] 2.1 Fix `record_conversation` in `api.rs` — use `Handle::try_current()` fallback instead of `Handle::current().block_on()`
- [x] 2.2 Fix `expect()` in `api_reg_cache.rs` — handle registry error gracefully
- [x] 2.3 Fix all remaining `unwrap_used`/`expect_used` violations across the codebase

## 3. Justfile recipe

- [x] 3.1 Add `check-lint-rules` recipe — parse Cargo.toml, verify `unwrap_used`, `expect_used`, `panic` are present in `[workspace.lints.clippy]`