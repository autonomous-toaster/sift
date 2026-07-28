## Context

Current dispatch flow for a command with variable expansion:

```
dispatch_full("echo \"$HOME\"")
  │
  └─ try_pipeline() → no pipe → None
  └─ normal dispatch
       ├─ shlex::split → ["echo", "$HOME"]
       ├─ find_plugin("echo", ["$HOME"]) → __default__ (bash.lua)
       └─ bash.lua: echo '$HOME'    ← LITERAL, $HOME not expanded
```

Target flow:

```
dispatch_full("echo \"$HOME\"")
  │
  └─ try_pipeline() → no pipe → None
  └─ normal dispatch
       ├─ shlex::split → ["echo", "$HOME"]
       ├─ find_plugin("echo", ["$HOME"]) → __default__
       └─ exec_command("echo \"$HOME\"")  ← ORIGINAL command, $HOME expands
```

## Goals / Non-Goals

**Goals:**
1. Commands containing `$` (variable expansion, command substitution) produce correct output
2. In-process plugins (jq, cat, head, tail, etc.) continue to work unchanged
3. Pipeline optimization continues to work unchanged
4. Passthrough path uses original command instead of reconstructed

**Non-Goals:**
- Handling every possible shell syntax edge case (e.g., nested subshells)
- Changing the Lua plugin API
- Modifying individual plugins

## Decisions

### Decision 1: Thread original command through dispatch

Add an `original_cmd: &str` parameter to:
- `dispatch_with_redirect()` → passes to `dispatch()`
- `dispatch()` → passes to `execute_passthrough()`
- `execute_passthrough()` → uses instead of reconstructing

### Decision 2: Bypass `__default__` plugin for original command

When `dispatch()` finds the `__default__` plugin, instead of calling the Lua plugin (which reconstructs the command lossily), run the original command via `exec_command()` directly. This preserves all shell semantics.

### Decision 3: Passthrough uses original command

When a plugin returns `status = "passthrough"`, `execute_passthrough()` uses the original command string instead of reconstructing from `cmd + args`.

## Risks / Trade-offs

- **Risk**: The `__default__` plugin (bash.lua) also handles stdin passthrough. When bypassing it, stdin handling must be preserved. Mitigation: `exec_command()` already accepts an optional stdin parameter.
- **Risk**: The `dispatch()` function is also called from `try_pipeline()`. In that context, the "original command" is the last pipeline segment, not the full pipeline. Mitigation: `try_pipeline()` passes the last segment as the original command, which is correct for that context.
- **Trade-off**: Bypassing the `__default__` Lua plugin means losing any future enhancements to bash.lua. Acceptable because bash.lua's only job is to run the command in bash, which `exec_command()` does directly.
