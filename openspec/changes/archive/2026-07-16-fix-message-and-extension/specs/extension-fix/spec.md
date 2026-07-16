# Pi extension fixes

## MODIFIED

### Shell quoting
- `JSON.stringify(path)` replaced with `shQuote()` (single-quote wrapping)
- `shQuote` wraps string in single quotes, escaping internal single quotes with `'\''`
- Applied to all shell command construction in the extension

### Session ID propagation
- Module-level `currentSessionId` variable, initialized to `"default"`
- `session_start` event handler gets real session ID from `ctx.sessionManager.getSessionId()`
- `spawnHook` uses `currentSessionId` for `AI_SESSION` env var
- Read tool execute function uses `currentSessionId` for `AI_SESSION` env var
- Reset cache handlers use `currentSessionId`

### Read tool via `createReadTool`
- Uses `createReadTool(cwd, { operations: { readFile, access } })` instead of manual `execSync`
- `readFile` calls `sift -c "sift-read <path>"` with proper env
- `access` delegates to `fs.access` (passthrough)
- No `bypass_cache` param, no `--fresh` logic — exact same interface as default read tool

## Verification

- `AI_SESSION` in spawned processes matches actual pi session ID
- Paths with spaces/special chars are properly quoted
- Read tool returns same output as default read tool for same inputs
