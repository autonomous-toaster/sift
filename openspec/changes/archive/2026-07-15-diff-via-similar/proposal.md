## Why

The `compute_diff` function in sift-read.lua is unused because we don't track `path → last_hash` to look up old content on cache miss. We need a proper diff system: track path-to-hash mappings, use the `similar` crate for unified diffs, and wire it into sift-read's cache miss path.

## What Changes

- **NEW**: `sift.diff(ctx, old, new)` — unified diff via `similar` crate, exposed to Lua.
- **NEW**: `sift.cache.set_path_hash(ctx, path, hash)` and `sift.cache.get_path_hash(ctx, path)` — file-based path-to-hash tracking.
- **MODIFIED**: `plugins/sift-read.lua` — on cache miss, look up old hash, load old content, compute diff, emit if useful.
- **REMOVED**: Unused `compute_diff` Lua function from sift-read.lua.

## Capabilities

### New Capabilities
- `diff-api`: `sift.diff(ctx, old, new)` returns unified diff string via `similar` crate.
- `path-hash-tracking`: `sift.cache.set_path_hash/get_path_hash` for looking up old content on cache miss.

### Modified Capabilities
- `sift-read-plugin`: On cache miss, emits unified diff when hash changes and diff is useful.

## Impact

- **sift-core/Cargo.toml**: Add `similar` dependency.
- **sift-core/src/lua/api.rs**: Add `sift.diff()`, `sift.cache.set_path_hash()`, `sift.cache.get_path_hash()`.
- **plugins/sift-read.lua**: Wire up diff on cache miss, remove unused `compute_diff`.
