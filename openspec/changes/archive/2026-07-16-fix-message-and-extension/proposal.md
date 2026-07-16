# Fix message format and pi extension

## What

Two small fixes:

1. **sift-read message format**: When `range_start == range_end`, say "line X" instead of "lines X-X"
2. **pi extension cleanup**: Fix shell quoting, propagate real session ID, use `createReadTool` with custom `ReadOperations`

## Why

- "lines 10-10" is ugly and confusing
- `JSON.stringify(path)` double-escapes quotes in shell commands
- `AI_SESSION` is always "default" — never the actual pi session ID
- `execSync` in read tool is blocking; should use pi's streaming read tool API
