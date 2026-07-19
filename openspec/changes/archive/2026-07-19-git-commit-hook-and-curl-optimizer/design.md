# Design: git-commit-hook and curl-json-optimizer plugins

## Context

Two new behavioral plugins for sift. Both are pure Lua — no core changes needed. They follow the existing plugin pattern (cat.lua, sift-read.lua, rtk.lua) and use the sift.* API.

## Goals / Non-Goals

**Goals:**
- `git-commit.lua` intercepts `git commit -n`/`--no-verify`, returns error + nudge
- `curl.lua` auto-detects JSON responses, compresses them, stores raw for re-read
- Both plugins passthrough for non-matching commands
- Tests use httpbin.org for curl behavior validation

**Non-Goals:**
- No core sift changes (no new sift.* API functions)
- No changes to existing plugins
- No new dependencies

## Decisions

### 1. Pattern matching: `"git commit"` works

The matching system builds candidates by appending args: `["git", "git commit", "git commit -m", ...]`. Pattern `"git commit"` matches exactly — no need to check `args[1]` in execute. Other git commands passthrough automatically.

### 2. Curl JSON detection via `-w` not `-v`

Using `curl -w "\n%{content_type}"` appends the response content type as the last line of stdout. This avoids needing `-v` (verbose headers) at all — no header parsing, no stripping verbose output. The body and content type are cleanly separated by the last newline.

**Alternatives considered:**
- `-D /dev/stderr` to dump headers to stderr → requires parsing stderr, fragile
- `-D -` to dump headers to stdout → requires parsing mixed output, complex
- `-w` approach → simplest, cleanest separation

### 3. No `-s` flag added

`-s` (silent) suppresses curl's progress meter. It's unrelated to JSON detection. The plugin only adds what's needed: `-w "\n%{content_type}"`. If the agent wants silent mode, they add `-s` themselves.

### 4. Raw JSON stored via `sift.store()`

Compressed JSON is returned to the agent. The raw JSON is written to `/tmp/sift/<session>/` via `sift.store()` and a nudge is emitted so the agent can re-read the original.

### 5. httpbin.org for tests

`https://httpbin.org/anything` returns JSON (request metadata). `https://httpbin.org/html` returns HTML. These provide predictable test fixtures without local servers.

## Risks / Trade-offs

- **`-w` output parsing**: If curl's stdout is empty (e.g., `-o` flag), the `-w` output is the only content. The `match("^(.*)\n([^\n]*)$")` pattern handles this — body is nil, content_type is the `-w` value. → Return output as-is.
- **False positive `-n` in commit message**: `git commit -m "fix -n issue"` would detect `-n` in args. → Track flags that take values (`-m`, `-F`, etc.) and skip their arguments.
- **httpbin availability**: Tests depend on httpbin.org being reachable. → Accept as test dependency; httpbin is a standard test service.

## Open Questions

None.
