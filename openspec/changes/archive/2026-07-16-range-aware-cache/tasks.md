## 1. Range cache API

- [x] 1.1 Add `sift.cache.add_range(ctx, hash, start, end)` with merge logic.
- [x] 1.2 Add `sift.cache.has_range(ctx, hash, start, end)` with union containment.

## 2. sift-read update

- [x] 2.1 Update sift-read.lua to use range-aware cache.

## 3. Cleanup

- [x] 3.1 Verify `just ci` passes.
