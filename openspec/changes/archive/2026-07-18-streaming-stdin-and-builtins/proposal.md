## Why

sift's plugin stdin API loads file content entirely into memory before passing it to plugins, causing OOM on large files. String utility functions (`split_lines`, `slice_text`, `is_sensitive`) are duplicated across 4 plugins, wasting ~80 lines of Lua. The pi extension silently swallows cache reset errors and resets cache on session events that don't need it (fork, switch, shutdown, tree navigation).

## What Changes

- **Streaming stdin**: Replace `stdin: Option<&str>` with a streaming Lua userdata (`StdinReader`) wrapping `BufReader<File>` or `Cursor<String>`. The Rust layer opens files and streams content; plugins read incrementally via `stdin:readline()`, `stdin:read(n)`, or `stdin:lines()`.
- **`sift.str.*` builtins**: Register `sift.str.split_lines()`, `sift.str.slice_text()`, `sift.str.is_sensitive()` in the Lua VM. Remove duplicated Lua implementations from head, tail, sed, sift-read, and cat plugins.
- **Extension error handling**: Replace silent `catch {}` with `ctx.ui.notify()` on success and error. Add cache hit detection to the reset plugin.
- **Session reset scoping**: Only reset cache on `session_compact`. Remove handlers for `session_shutdown`, `session_tree`, `session_fork`, `session_switch` — cache is naturally isolated by session ID.

## Capabilities

### New Capabilities
- `streaming-stdin`: Streaming stdin reader for Lua plugins — Rust opens files, plugins read incrementally
- `str-builtins`: Shared string utility functions registered as `sift.str.*` in the Lua VM
- `extension-notifications`: User-visible notifications for cache reset success/failure with cache hit detection

### Modified Capabilities
- (none — no existing spec requirements change)

## Impact

- `sift-core/src/lua/api.rs`: `dispatch()` and `dispatch_full()` — change stdin parameter type, add `stdin_path` context field
- `sift-core/src/lua/api_reg_io.rs`: Add `sift.str.*` registration
- `sift-core/src/lua/mod.rs`: Add `StdinReader` userdata type
- `plugins/*.lua`: Remove duplicated `split_lines`, `slice_text`, `is_sensitive` — use `sift.str.*` instead
- `integrations/pi/sift.ts`: Fix error handling, add notifications, remove unnecessary session reset handlers
- `sift/plugins/reset.lua`: Add cache hit detection output
