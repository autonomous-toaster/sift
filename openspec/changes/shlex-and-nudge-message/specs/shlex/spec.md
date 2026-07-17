# Replace manual shell parsing with shlex crate

## MODIFIED

### dispatch_full() in api.rs

- Add `shlex = "2"` dependency to `sift-core/Cargo.toml`
- Replace `split_whitespace()` + `strip_quotes()` with `shlex::split()` in `dispatch_full()`
- On `shlex::split()` returning `None`, fall back to `split_whitespace()`
- Remove the `strip_quotes()` helper function
- Update pipeline segment parsing to use `shlex::split()` too

## Verification

- `sift-read '/path/with spaces/file'` works (quoted path with spaces)
- `sift-read "file with spaces.txt"` works (double-quoted path)
- `sift-read file` works (unquoted path)
- Malformed input falls back to `split_whitespace()` gracefully
- All existing tests pass
