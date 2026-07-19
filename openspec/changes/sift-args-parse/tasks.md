## 1. Rust implementation

- [x] 1.1 Create `sift-core/src/lua/api_reg_args.rs` with `register_args()` and the `sift.args.parse()` function
- [x] 1.2 Wire `register_args()` into `sift-core/src/lua/mod.rs` and `sift-core/src/lua/api.rs`
- [x] 1.3 Add unit tests for `sift.args.parse()` covering: boolean flags, int/str flags, short count, combined short flags, long flags with `=`, `--` end-of-flags, unknown flags, missing required positional, extra positional, type coercion errors

## 2. Plugin conversion

- [x] 2.1 Convert cat.lua — replace manual flag check with `sift.args.parse()`
- [x] 2.2 Convert head.lua — replace `parse_head()` with `sift.args.parse()`
- [x] 2.3 Convert tail.lua — replace `parse_tail()` with `sift.args.parse()`
- [x] 2.4 Convert sed.lua — replace `parse_sed_range()` with `sift.args.parse()`
- [x] 2.5 Convert sift-read.lua — replace manual index tracking with `sift.args.parse()`
- [x] 2.6 Convert git-commit.lua — replace manual flag scanner with `sift.args.parse()`
- [x] 2.7 Convert curl.lua — replace manual presence check with `sift.args.parse()`
- [x] 2.8 Convert openspec.lua — replace manual presence check with `sift.args.parse()`
