# Bypass Mechanism

## Task Reference

| Task ID | Description |
|---------|-------------|
| T6.1 | Implement `command.lua` built-in plugin with priority 1000 |
| T6.2 | Implement passthrough dispatch: run real binary when plugin returns passthrough |

## Requirements

### Requirement: command plugin precedes passthrough dispatch

T6.1 SHALL complete BEFORE T6.2 SHALL run.

#### Scenario: Passthrough depends on command plugin

- **WHEN** T6.1 registers the command plugin
- **THEN** T6.2 SHALL handle passthrough results by running the real binary.

### Requirement: command plugin matches "command" prefix

ALWAYS T6.1 SHALL match any command starting with "command".

#### Scenario: command cat foo

- **WHEN** a user runs "command cat foo"
- **THEN** T6.1 SHALL match and return `{status="passthrough"}`.

### Requirement: Passthrough runs real binary

ALWAYS T6.2 SHALL execute the real binary when a plugin returns passthrough.

#### Scenario: Real cat runs

- **WHEN** T6.2 receives a passthrough result for "cat foo"
- **THEN** T6.2 SHALL execute `/bin/cat foo` and emit the raw output.

### Requirement: Passthrough bypasses all plugins

ALWAYS T6.2 SHALL NOT run any other plugins once passthrough is received.

#### Scenario: No plugin chain

- **WHEN** T6.2 receives passthrough
- **THEN** T6.2 SHALL skip all remaining plugin dispatch and run the real binary directly.
