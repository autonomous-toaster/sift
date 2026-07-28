## Context

### Current flow for unmatched commands

```
dispatch() → find_entry() → __default__ or *
  → BYPASS Lua (check in dispatch())
  → exec_command() in Rust (bash -c "...")
  → bash plugin's execute() is NEVER called
```

### Target flow

```
dispatch() → find_entry() → __default__
  → call shell plugin's execute() in Lua
  → plugin uses ctx.original_cmd (preserves shell semantics)
  → plugin calls sift.exec(ctx, ctx.original_cmd)
  → exec_command() in Rust (bash -c "...")
```

### For rtk-matched commands

```
dispatch() → find_entry() → rtk plugin (narrowed patterns)
  → call rtk plugin's execute() in Lua
  → plugin runs rtk <cmd>
  → on failure, returns passthrough → execute_passthrough() → exec_command()
```

## Goals / Non-Goals

**Goals:**
1. Remove the `__default__` and `*` bypass in `dispatch()`
2. Make bash.lua the active, overridable shell plugin
3. Add `ctx.original_cmd` to the plugin context
4. Narrow rtk patterns to only commands rtk handles
5. All existing tests pass

**Non-Goals:**
- Adding sandboxing or VM integration (future work)
- Changing how specific plugins (cat, git, curl) work
- Changing the `command` passthrough mechanism

## Decisions

### Decision 1: bash.lua stays as the built-in shell plugin
Instead of creating a new plugin, we update the existing `bash.lua`. Its `execute()` will use `ctx.original_cmd` instead of shell-quoting. This minimizes changes.

### Decision 2: `ctx.original_cmd` is the full original command string
The context table gets a new field `original_cmd` containing the exact command string as typed by the agent. This is the same string already threaded through `dispatch_with_redirect()` → `dispatch()` → `execute_passthrough()`.

### Decision 3: rtk patterns are a list of known commands
rtk's `pattern = "*"` is replaced with a list of ~30 commands. This is a one-time mapping that matches rtk's `--help` output.

### Decision 4: Pipeline fallback dispatches through shell plugin
When `try_pipeline()` finds no matching plugin for the last segment, instead of calling `exec_command()` directly, it dispatches the full pipeline through `dispatch()` with the pipeline string as the command. This ensures the shell plugin is the single canonical execution path.

### Decision 5: `*` wildcard is no longer special
With rtk narrowed, no plugin uses `*`. The `*` bypass in `dispatch()` is removed. If a future plugin uses `*`, it will go through the normal plugin execution path.

## Risks / Trade-offs

- **Risk**: A user plugin with `pattern = "*"` could intercept all commands. Mitigation: this is the user's choice, and they can always use `command` prefix to bypass.
- **Risk**: Performance regression from extra Lua call. Mitigation: one Lua function call per unmatched command is negligible (~microseconds).
- **Risk**: rtk pattern list drifts from rtk's actual supported commands. Mitigation: document that the list should match `rtk --help`.
