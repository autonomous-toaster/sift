## Why

Sift's output optimization (caching, diff emission, output compression) saves tokens but introduces correctness bugs: heredoc syntax corrupts files, diff emission causes the agent to make wrong edits, and stale cache entries cause the agent to work with outdated content. These bugs cost more tokens in retries and error recovery than the optimizations save.

## What Changes

- **cat.lua**: Fix heredoc crash — validate path before stat, passthrough to bash on invalid paths
- **sift-read.lua**: Remove diff emission — always return full content on file change, add one-line change notification
- **sift-core**: Make `sift.fs.stat()` return nil instead of raising Lua error on non-existent paths
- **sift-core**: Add mtime-based cache invalidation — force re-read if file mtime > last_read
- **sift-core**: Invalidate path cache on `sift.fs.write()` and `sift.fs.edit()`
- **sift-core**: Boost `command` plugin priority to ensure bypass always works

## Capabilities

### New Capabilities
- `agent-reliability`: Sift plugin and core fixes that prevent agent errors caused by output optimization — heredoc safety, full-content reads on change, cache freshness, and reliable command bypass

### Modified Capabilities
None — these are internal sift fixes, no spec-level behavior changes.

## Impact

- `baish/plugins/cat.lua` — heredoc validation
- `baish/plugins/sift-read.lua` — remove diff emission
- `baish/sift-core/src/lua/api_reg_io.rs` — `sift.fs.stat()` return nil on error
- `baish/sift-core/src/lua/api_reg_cache.rs` — mtime-based invalidation, write invalidation
- `baish/sift-core/src/lua/api.rs` — plugin priority for `command`
