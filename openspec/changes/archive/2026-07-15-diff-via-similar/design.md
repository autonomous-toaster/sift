## Context

sift-read.lua has an unused `compute_diff` function because we don't track `path → last_hash`. Adding path-to-hash tracking and a Rust-side diff function (via `similar` crate) enables proper diff-on-cache-miss behavior.

## Goals / Non-Goals

**Goals:**
- `sift.diff(ctx, old, new)` returns unified diff via `similar`
- `sift.cache.set_path_hash/get_path_hash` for path-to-hash tracking
- sift-read emits diff on cache miss when useful

**Non-Goals:**
- No changes to cat.lua (it doesn't need diff)
- No changes to the usefulness gate logic (stays in plugin)

## Decisions

### D1 — similar crate

```rust
use similar::TextDiff;

fn sift_diff(old: &str, new: &str) -> String {
    TextDiff::from_lines(old, new)
        .unified_diff()
        .context_radius(3)
        .to_string()
}
```

### D2 — Path hash storage

Files at `/tmp/sift/<session>/paths/<sha256(path)>` containing the content hash as plain text.

### D3 — Usefulness gate in plugin

```lua
local diff = sift.diff(ctx, old_content, content)
if #diff < #content * 0.9 then
    return { status = "handled", output = diff }
end
-- fall through to full content
```

## Risks / Trade-offs

- **similar crate**: Small, well-maintained, no transitive deps. Adds ~0.1s compile time.
- **Path hash files**: One file per unique path. Cleaned up by the same TTL mechanism as cache entries.
