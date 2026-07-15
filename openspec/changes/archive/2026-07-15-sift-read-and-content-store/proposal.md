## Why

sift needs a `read` plugin that shares cache state with the existing `cat` plugin, so `cat file.txt` then `read file.txt:5-10` knows the file hasn't changed (and vice versa). This replaces pi-readcache's functionality with a simpler, shared cache layer. The content store must be clearable — automatically via pruning, and explicitly via the reset plugin.

## What Changes

- **NEW**: Content-addressed store — `sift.cache.store(hash, content)` and `sift.cache.load(hash)` for persisting file content by sha256 hash.
- **NEW**: `plugins/sift-read.lua` — reads files with offset/limit, uses hash-based caching shared with cat plugin, supports slice comparison for range reads, emits unified diffs when full file changes, and accepts `--fresh` flag to bypass cache.
- **MODIFIED**: `plugins/cat.lua` — uses content store for cross-plugin cache sharing.
- **MODIFIED**: `plugins/reset.lua` — clears the content store on reset.
- **NEW**: Automatic pruning of objects older than a configurable max age.

## Capabilities

### New Capabilities
- `content-store`: Content-addressed storage by sha256 hash — `sift.cache.store(hash, content)` and `sift.cache.load(hash)`.
- `sift-read-plugin`: A `sift-read` plugin that reads files with offset/limit, shares cache with cat plugin, compares slices for range reads, and emits diffs.
- `store-cleanup`: Automatic pruning of old objects + explicit clearing via reset plugin.

### Modified Capabilities
- `cat-plugin`: Updated to use content store for cross-plugin cache sharing.
- `reset-plugin`: Updated to clear the content store.

## Impact

- **sift-core/src/lua/api.rs**: Add `sift.cache.store()` and `sift.cache.load()` bindings.
- **sift-core/src/session.rs**: Add content store (objects dir by hash).
- **plugins/sift-read.lua**: New plugin.
- **plugins/cat.lua**: Minor update to use content store.
- **plugins/reset.lua**: Clear content store on reset.
