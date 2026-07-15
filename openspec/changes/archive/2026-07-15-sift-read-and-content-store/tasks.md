## 1. Content-addressed store

- [x] 1.1 Add `sift.cache.store(hash, content)` — persist content by hash in `/tmp/sift/<session>/objects/`.
- [x] 1.2 Add `sift.cache.load(hash)` — load content by hash from objects dir.

## 2. sift-read plugin

- [x] 2.1 Create `plugins/sift-read.lua` with hash-based caching, offset/limit, slice comparison, diff emission.

## 3. cat plugin update

- [x] 3.1 Update `plugins/cat.lua` to use content store for cross-plugin cache sharing.

## 4. Store cleanup

- [x] 4.1 Add automatic pruning of objects older than 24h.
- [x] 4.2 Update `plugins/reset.lua` to clear the content store.

## 5. Cleanup

- [x] 5.1 Verify `just ci` passes with all changes.
