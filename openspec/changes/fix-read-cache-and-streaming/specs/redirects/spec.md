# Redirect handling in dispatch_full

## ADDED

### Redirect parsing in `dispatch_full()`

- Parse `< file` from args: read file content, pass as stdin to plugin, strip `<` and file from args
- Parse `> file` from args: capture plugin output, write to file, strip `>` and file from args
- Parse `>> file` from args: capture plugin output, append to file, strip `>>` and file from args
- Complex redirects (`2>`, `&>`, heredocs, `<<<`) fall through to the shell
- Redirect parsing happens after pipeline handling, before normal dispatch

## Verification

- `sed -n '1,10p' < Justfile` returns lines 1-10 (no crash)
- `echo hello > /tmp/out` writes "hello" to /tmp/out
- `echo hello >> /tmp/out` appends "hello" to /tmp/out
- `echo hello 2> /tmp/err` falls through to shell (stderr redirect not handled)
