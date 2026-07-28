## Why

Currently, sift has two parallel execution paths for commands that don't match a specific plugin:

1. **Rust bypass**: `dispatch()` checks if the matching plugin is `__default__` or `*`, and if so, calls `exec_command()` directly — bypassing Lua entirely.
2. **Lua plugin**: The bash plugin (`sift/plugins/bash.lua`) has an `execute()` function that is **never called** because the bypass always triggers first.

This means:
- The user **cannot override** how commands are executed (sandboxing, env injection, containerization)
- The bash plugin is dead code — misleading to future developers
- The `*` wildcard (used by rtk) is also bypassed, but rtk's broad pattern causes issues

## What Changes

1. **Narrow rtk patterns**: Change `pattern = "*"` to a list of commands rtk actually handles. Commands not in the list go directly to the shell plugin.

2. **Remove the `__default__`/`*` bypass**: `dispatch()` always calls the matching plugin's `execute()`. No more Rust-side bypass.

3. **Make bash.lua the active shell plugin**: Update it to use `ctx.original_cmd` (preserving shell semantics) instead of shell-quoting args.

4. **Add `ctx.original_cmd`**: The context table passed to plugins includes the original command string.

5. **User overridable**: A user can create `~/.config/sift/plugins/shell.lua` with pattern `__default__` to override command execution.

## Capabilities

### New Capabilities
- `shell-plugin-override`: Users can override the shell plugin to customize command execution (sandboxing, env injection, etc.)

### Modified Capabilities
- `plugin-context`: The `ctx` table now includes `original_cmd` — the full original command string
- `rtk-plugin`: Pattern narrowed from `*` to specific commands
- `bash-plugin`: Now actively used instead of being dead code
