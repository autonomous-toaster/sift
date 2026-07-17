# Tasks

## 1. Replace manual parsing with `shlex`

- [x] Add `shlex = "2"` to `sift-core/Cargo.toml`
- [x] Replace `split_whitespace()` + `strip_quotes()` with `shlex::split()` in `dispatch_full()`
- [x] Update pipeline segment parsing to use `shlex::split()`
- [x] Remove `strip_quotes()` helper function
- [x] Add fallback to `split_whitespace()` on parse error
- [x] Run tests to verify

## 2. Fix nudge message format

- [ ] Update `plugins/sift-read.lua` unchanged message to single-line format with "bypass if stale"
- [ ] Update `plugins/cat.lua` unchanged message to single-line format
- [ ] Update `plugins/sed.lua` unchanged message to single-line format
- [ ] Update `plugins/head.lua` unchanged message to single-line format
- [ ] Update `plugins/tail.lua` unchanged message to single-line format
- [x] Update diff header to `[sift: N lines changed of M]` (no bypass nudge)
- [x] Run tests to verify
