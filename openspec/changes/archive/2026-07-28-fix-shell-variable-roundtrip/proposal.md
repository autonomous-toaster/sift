## Why

When a shell command contains variable expansion (`$VAR`, `${VAR}`), command substitution (`$(cmd)`), or other shell metacharacters, sift's plugin dispatch system destroys them. The flow is:

1. `dispatch_full()` splits the command string into `cmd` + `args` via `shlex::split()`
2. A plugin (bash.lua fallback or rtk wildcard) reconstructs the command by shell-quoting each arg
3. Shell-quoting turns `$HOME` into `'$HOME'` — a literal string, not a variable reference
4. Bash receives the literal string, no expansion occurs

This affects any command using `$` — from simple `echo "$HOME"` to `curl "$URL"` to `mv "${SRC}" "${DST}"`. The plugin system's command reconstruction is lossy for shell metacharacters.

## What Changes

Thread the **original full command string** through the dispatch pipeline so that when a plugin returns "passthrough" or the `__default__` fallback is used, the original command is run in bash instead of a reconstructed (lossy) version.

Specifically:
- `dispatch_full()` passes the original command to `dispatch_with_redirect()` and `dispatch()`
- `dispatch()` passes it to `execute_passthrough()`
- `execute_passthrough()` uses the original command instead of reconstructing from `cmd + args`
- When the `__default__` plugin matches, `dispatch()` runs the original command via `exec_command()` instead of calling the Lua plugin

## Capabilities

### New Capabilities
- `shell-variable-roundtrip`: Shell variable expansion and other `$`-based constructs survive the sift plugin dispatch round-trip.

### Modified Capabilities
*(No existing capability specs are changing.)*

## Impact

- **sift-core/src/lua/api.rs**: `dispatch_full()`, `dispatch_with_redirect()`, `dispatch()`, `execute_passthrough()` — thread original command through
- **sift-core/src/lua/tests.rs**: New tests for variable expansion in commands
