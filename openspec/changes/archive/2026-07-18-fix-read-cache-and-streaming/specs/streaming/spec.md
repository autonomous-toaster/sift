# Output duplication: bash plugin stops returning `output`

## MODIFIED

### bash.lua plugin

- Remove `output` field from return value: `{ status = "handled", exit_code = 0 }`
- Output is already streamed by `sift.exec()` → `exec_command()` writing to real stdout
- `dispatch()` already handles missing `output` via `result.get("output").unwrap_or_default()`

### dispatch() in api.rs

- No code change needed — existing `if !output.is_empty()` guard handles empty output
- Other plugins (sift-read, cat, sed, head, tail) continue returning `output` as before

## Verification

- `bash("echo hello")` outputs "hello" once (not twice)
- `bash("wc -l < Justfile")` outputs line count once (not duplicated)
- `sift-read /path` still returns content correctly
- Pipeline optimization still works (preceding stdout passed as stdin)
