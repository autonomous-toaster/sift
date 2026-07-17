# spawnHook: JSON.stringify instead of shQuote

## MODIFIED

### spawnHook in sift.ts

- Replace `shQuote(command)` with `JSON.stringify(command)` in spawnHook
- `JSON.stringify` is safe: no `/` escaping in Node.js, `$`/`` ` `` expansion inside double quotes is desired
- `<`/`>` are literal inside double quotes (redirects handled by `dispatch_full`)
- Single quotes in command are literal (no double-quoting bug)

### siftExec function

- Already uses `JSON.stringify` (from previous fix) — no change needed

## Verification

- `bash("cat Justfile")` works correctly
- `bash("echo $HOME")` expands `$HOME` correctly
- `bash("sed -n '1,10p' < Justfile")` works (redirect handled by `dispatch_full`)
- No double-quoting bug with paths containing single quotes
