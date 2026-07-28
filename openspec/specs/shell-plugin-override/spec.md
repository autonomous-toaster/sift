# Shell Plugin Override

## Purpose

Allow users to override the built-in shell plugin (`__default__`) to customize command execution (sandboxing, env injection, containerization, etc.).

## Requirements

### Requirement: Shell plugin SHALL be the canonical execution path

When no specific plugin matches a command, `dispatch()` SHALL call the `__default__` plugin's `execute()` instead of bypassing Lua.

#### Scenario: dispatch calls shell plugin for unmatched commands

- **WHEN** no specific plugin matches a command
- **THEN** `dispatch()` SHALL call the `__default__` plugin's `execute()`

### Requirement: Shell plugin SHALL use `ctx.original_cmd`

The `__default__` plugin's `execute()` SHALL use `ctx.original_cmd` to preserve shell semantics (variable expansion, command substitution, heredocs) instead of shell-quoting individual args.

#### Scenario: shell plugin preserves shell semantics

- **WHEN** the shell plugin executes a command
- **THEN** it SHALL use `ctx.original_cmd` to preserve shell semantics

### Requirement: Pipeline fallback SHALL dispatch through shell plugin

When `try_pipeline()` finds no matching plugin for the last segment, it SHALL dispatch the full pipeline through `self.dispatch()` instead of calling `exec_command()` directly.

#### Scenario: pipeline fallback goes through shell plugin

- **WHEN** a pipeline's last segment has no matching plugin
- **THEN** `try_pipeline()` SHALL dispatch the full pipeline through `self.dispatch()`

### Requirement: User SHALL be able to override the shell plugin

A user plugin with pattern `__default__` SHALL override the built-in shell plugin. User plugins are loaded after built-in plugins and take precedence at the same priority.

#### Scenario: User overrides shell plugin

- **WHEN** a user creates `~/.config/sift/plugins/shell.lua` with `pattern = "__default__"`
- **THEN** all unmatched commands SHALL be dispatched to the user's plugin instead of the built-in shell plugin
