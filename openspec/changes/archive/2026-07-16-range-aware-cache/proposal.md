## Why

The current cache uses the full file hash as the sole cache key. When the agent reads lines 1-2, then asks for lines 1-4, the cache returns "unchanged" — but the agent never saw lines 3-4. The cache must track which line ranges the agent has actually read.

## What Changes

- **NEW**: Cache marker JSON stores `ranges: [[start,end], ...]` — the set of line ranges the agent has read for this file hash.
- **NEW**: `sift.cache.add_range(ctx, hash, start, end)` — adds a range, merging overlapping/adjacent ranges.
- **NEW**: `sift.cache.has_range(ctx, hash, start, end)` → bool — true if the union of cached ranges fully contains the requested range.
- **MODIFIED**: `plugins/sift-read.lua` — checks `has_range()` before declaring "unchanged"; stores ranges on cache miss.

## Capabilities

### New Capabilities
- `range-cache`: Range-aware cache with merge-on-add and union containment check.

### Modified Capabilities
- `sift-read-plugin`: Uses range-aware cache for correct "unchanged" detection.

## Impact

- **sift-core/src/lua/api.rs**: Add `add_range` and `has_range` Rust functions.
- **plugins/sift-read.lua**: Update cache check and store logic.
