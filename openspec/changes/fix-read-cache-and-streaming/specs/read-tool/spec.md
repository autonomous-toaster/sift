# Read tool: custom execute function

## ADDED

### Custom read tool replacing `createReadTool`

- Replace `createReadTool` with a custom tool definition with its own `execute` function
- `execute` calls `siftExec("sift-read " + shQuote(path))`, sift resolves path internally
- No `bypass_cache` param, no `--fresh` logic — same interface as default read tool
- Image files handled by reading directly (same as pi-readcache)
- The agent receives marker/diff/content from `sift-read` and is expected to understand it

## Verification

- `read(path="Justfile")` returns file content on first read
- `read(path="Justfile")` returns `[sift] file unchanged` on cache hit
- `read(path="Justfile", offset=10, limit=5)` returns sliced content or range marker
