## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Remove `__default__`/`*` bypass in `dispatch()` |
| T3.2 | Update `bash.lua` to use `ctx.original_cmd` |
| T3.3 | Update pipeline fallback to dispatch through shell plugin |
| T3.4 | Verify `FILE=value echo test` preserves shell semantics |
| T3.5 | Verify `ls -la` runs through shell plugin |
| T3.6 | Verify `echo 1 \| head` runs through shell plugin |
| T3.7 | Verify all existing tests pass |

## ADDED Requirements

### Requirement: Shell plugin SHALL be the canonical execution path

T3.1 SHALL complete BEFORE T3.2 SHALL start.

T3.1 SHALL complete BEFORE T3.3 SHALL start.

#### Scenario: dispatch calls shell plugin for unmatched commands

**WHEN** T3.1 completes
**THEN** `dispatch()` SHALL call the `__default__` plugin's `execute()` instead of bypassing Lua.

### Requirement: Shell plugin SHALL use `ctx.original_cmd`

T3.2 SHALL complete AFTER T2.1 SHALL complete.

#### Scenario: shell plugin preserves shell semantics

**WHEN** T3.2 completes
**THEN** the shell plugin's `execute()` SHALL use `ctx.original_cmd` instead of shell-quoting individual args.

### Requirement: Pipeline fallback SHALL dispatch through shell plugin

T3.3 SHALL complete AFTER T3.1 SHALL complete.

#### Scenario: pipeline fallback goes through shell plugin

**WHEN** T3.3 completes
**THEN** `try_pipeline()` SHALL dispatch the full pipeline through `self.dispatch()` instead of calling `exec_command()` directly.

### Requirement: User SHALL be able to override the shell plugin

T3.1 SHALL complete BEFORE T3.2 SHALL start.

#### Scenario: User overrides shell plugin

**WHEN** a user creates `~/.config/sift/plugins/shell.lua` with `pattern = "__default__"`
**THEN** all unmatched commands SHALL be dispatched to the user's plugin instead of the built-in shell plugin.
