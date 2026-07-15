## Context

sift needs a `read` plugin that shares cache state with the existing `cat` plugin. Both compute sha256 of full file content and use the same session store. A content-addressed store preserves old content for slice comparison and diff emission.

## Goals / Non-Goals

**Goals:**
- `sift-read` plugin with offset/limit, hash caching, slice comparison, diff emission
- Shared cache between `cat` and `sift-read` (same session store)
- Content-addressed store for old content preservation
- Automatic pruning + reset plugin clears store

**Non-Goals:**
- No pi extension yet (prepared for, but not implemented)
- No write/edit tools (hash detects any change)

## Decisions

### D1 — Content-addressed store

Objects stored at `/tmp/sift/<session>/objects/sha256-<hash>.txt`. Two new `sift.cache` methods:

```lua
sift.cache.store(ctx, hash, content)  -- persists content by hash
sift.cache.load(ctx, hash)            -- returns content or nil
```

### D2 — sift-read plugin

Pattern: `sift-read`. Args: `<path> [<offset> [<limit>]]`.

Flow:
1. Read full file via `sift.fs.read()`
2. Compute sha256
3. Check `sift.cache.has(hash)`:
   - YES → return "unchanged" (full or range)
   - NO → load old content by old hash (if available)
     - Range read, slice matches → "unchanged; changes outside range"
     - Range read, slice differs → return range content
     - Full read → compute diff, emit if useful, else return full content
4. Store new content, cache new hash

### D3 — Diff emission

Use Rust-side unified diff (via `diff` crate). Only emit if diff is < 90% of full content size (same gate as pi-readcache).

### D4 — Store cleanup

Pruning runs on session start (via `sift.meta` or startup). Max age: 24h. Reset plugin deletes the entire objects directory for the session.

## Risks / Trade-offs

- **Storage**: Content-addressed store grows with file versions. Pruning mitigates this.
- **Diff crate**: New dependency. Small, well-maintained.
- **Slice comparison**: Requires loading old content from disk. Acceptable — only happens on cache miss.
