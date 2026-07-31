## 1. sift.fs.stat return nil on error

- [x] 1.1 Change `sift.fs.stat()` in `sift-core/src/lua/api_reg_io.rs` to return `Option<Table>` — `Ok(None)` on error instead of raising
- [x] 1.2 Update `sift.fs.stat()` callers in cat.lua and sift-read.lua to handle `nil` return

## 2. cat plugin heredoc safety

- [x] 2.1 Add path validation in cat.lua — check for shell metacharacters before calling `sift.fs.stat()`
- [x] 2.2 Add `sift.fs.exists()` check as fallback — passthrough if path doesn't exist
- [x] 2.3 Test: heredoc syntax (`cat << 'EOF'`) passthrough to bash
- [x] 2.4 Test: non-existent file passthrough instead of crash

## 3. sift-read diff emission removal

- [x] 3.1 Remove diff emission block from sift-read.lua (lines ~130-160)
- [x] 3.2 Add one-line change notification: `sift.nudge(ctx, "<filename> changed since last read")`
- [x] 3.3 Ensure full content is always returned on cache miss with content change
- [x] 3.4 Test: file changed since last read returns full content, not diff
- [x] 3.5 Test: file unchanged still returns "unchanged"

## 4. Cache invalidation on write

- [x] 4.1 Add `sift.cache.invalidate_path(path)` function in `sift-core/src/lua/api_reg_cache.rs`
- [x] 4.2 Call `sift.cache.invalidate_path()` in `sift.fs.write()` implementation
- [x] 4.3 Call `sift.cache.invalidate_path()` in `sift.fs.edit()` implementation
- [x] 4.4 Test: write then read returns new content, not "unchanged"
- [x] 4.5 Test: edit then read returns new content, not "unchanged"

## 5. mtime-based cache staleness

- [x] 5.1 Add mtime tracking to file cache entries in `sift-core/src/lua/api_reg_cache.rs`
- [x] 5.2 Add mtime check before returning "unchanged" in sift-read.lua
- [x] 5.3 Add mtime check before returning "unchanged" in cat.lua
- [x] 5.4 Test: file modified externally triggers re-read
- [x] 5.5 Test: unchanged file still returns "unchanged"

## 6. command plugin priority boost

- [x] 6.1 Set `command` plugin priority higher than all user plugins in `sift-core/src/lua/api.rs`
- [x] 6.2 Test: `command cat <path>` bypasses cat plugin
- [x] 6.3 Test: `command sift-read <path>` bypasses sift-read plugin
