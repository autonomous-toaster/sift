## Why

sift replaces the default bash tool in pi coding agent. The current `exec_command()` uses `std::process::Command::output()` which buffers all output until the process exits. For long-running commands (builds, tests, installs), the user sees nothing until completion. Streaming output to stdout/stderr in real-time is non-negotiable for a shell proxy.

## What Changes

- **NEW**: `exec_command()` spawns the process, reads stdout/stderr in 4KB chunks, writes each chunk to the real stdout/stderr immediately, and collects into a buffer for the return value.
- **NEW**: `sift.exec()` accepts an optional `{ transform = function(chunk) end }` option for plugins that want to modify output chunks in-flight before they're written to stdout.
- **MODIFIED**: All dispatch paths (plugin, passthrough, pipeline) stream automatically since they all go through `exec_command()`.

## Capabilities

### New Capabilities
- `streaming-exec`: `exec_command()` streams stdout/stderr to the real stdout/stderr in real-time while collecting for the return value.
- `streaming-transform`: `sift.exec()` accepts `{ transform = fn }` to modify each output chunk before writing and collecting.

## Impact

- **sift-core/src/lua/mod.rs**: `exec_command()` rewritten to spawn + read chunks + write + collect. Signature unchanged.
- **sift-core/src/lua/api.rs**: `sift.exec()` Lua binding updated to accept optional `{ transform = fn }` parameter.
- **No change to plugins**: All existing plugins work unchanged. Streaming is transparent.
- **No change to agent_mode()**: Output is already streamed inside `exec_command()`.
