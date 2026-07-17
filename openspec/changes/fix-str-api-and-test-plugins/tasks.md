## 1. Fix sift.str.* API signatures

- [x] 1.1 Remove `ctx` parameter from `sift.str.split_lines()` Rust closure signature
- [x] 1.2 Remove `ctx` parameter from `sift.str.slice_text()` Rust closure signature
- [x] 1.3 Remove `ctx` parameter from `sift.str.is_sensitive()` Rust closure signature

## 2. Add plugin unit tests

- [ ] 2.1 Create `tests_plugins.rs` with smoke test loading all `.lua` files and verifying full `sift.*` API visibility
- [ ] 2.2 Add per-plugin execution tests for sift-read, cat, head, tail, sed with fixture files
- [ ] 2.3 Wire `tests_plugins.rs` into the module tree and verify all tests pass

## 3. Fix clippy violations

- [x] 3.1 Fix `uninlined_format_args` in `api.rs` and `stdin_reader.rs`
- [x] 3.2 Fix `option_if_let_else` in `api_reg_cache.rs` (has_any)
- [x] 3.3 Fix `cast_possible_truncation` in `api_reg_io.rs` (slice_text) and `stdin_reader.rs` (read method)
- [x] 3.4 Fix `significant_drop_tightening` in `stdin_reader.rs` (read method)
- [x] 3.5 Fix `use_self` in `stdin_reader.rs` (lines method)
- [x] 3.6 Fix `doc_markdown` in `mod.rs`
- [x] 3.7 Fix `too_many_lines` in `api.rs` (dispatch_full) — extract pipeline and redirect helpers

## 4. Update README

- [ ] 4.1 Update `sift.str.*` API reference in README to document pure-function signatures without `ctx`