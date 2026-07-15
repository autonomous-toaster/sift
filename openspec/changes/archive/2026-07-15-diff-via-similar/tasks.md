## 1. Diff API

- [x] 1.1 Add `similar` crate and expose `sift.diff(ctx, old, new)` returning unified diff.

## 2. Path hash tracking

- [x] 2.1 Add `sift.cache.set_path_hash(ctx, path, hash)` and `sift.cache.get_path_hash(ctx, path)`.

## 3. sift-read diff wiring

- [x] 3.1 Wire up diff in sift-read.lua: on cache miss, look up old hash, load old content, compute diff, emit if useful. Remove unused `compute_diff`.

## 4. Cleanup

- [x] 4.1 Verify `just ci` passes.
