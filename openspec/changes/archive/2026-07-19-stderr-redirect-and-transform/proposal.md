## Why

When a user runs `curl http://api.example.com 2>&1`, bash merges stderr into stdout at the shell level before sift can apply transforms. Progress bars and errors get fed through JSON→TOON compression, producing garbage. Sift needs to intercept fd redirects at the arg parsing stage, keep streams separate, apply transforms to stdout only, then append stderr raw.

## What Changes

- Add `parse_fd_redirects()` — detect all `N>&M` patterns in args (e.g., `2>&1`, `1>&2`), strip them, return a merge map
- Handle `2>&1` in `dispatch_with_redirect()` — strip from args, set `merge_stderr` flag
- Add `merge_stderr` parameter to `exec_command()` — keep stdout/stderr separate, apply transform to stdout only, append stderr
- Add `merge_stderr` option to `sift.exec()` Lua function — pass through to `exec_command`
- Fix `bash.lua` to return `output` in its result table (currently relies on exec_command printing side effects)
- Pipeline deferral: if `|` detected, skip all redirect handling — let bash handle it
- `> file` / `>> file` behavior unchanged — file gets whatever the plugin outputs (transformed or not)

## Capabilities

### New Capabilities
- (none)

### Modified Capabilities
- `sift-api`: `sift.exec` gains `merge_stderr` option
- `plugin-tests`: Update tests for bash.lua returning output

## Impact

- **sift-core/src/lua/exec.rs**: Add `merge_stderr` param to `exec_command`
- **sift-core/src/lua/api_reg_cache.rs**: Pass `merge_stderr` from options to `exec_command`
- **sift-core/src/lua/api.rs**: Add `parse_fd_redirects()`, detect `N>&M` patterns in `dispatch_with_redirect`, strip from args, set flags
- **sift/plugins/bash.lua**: Return `output` in result table
- **sift-core/src/lua/tests.rs**: Update exec_command tests
- **sift-core/src/lua/tests_plugins.rs**: Update bash.lua tests