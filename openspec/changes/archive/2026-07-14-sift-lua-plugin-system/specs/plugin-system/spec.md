# Plugin System

## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Implement plugin registry with priority-based resolution and longest-prefix matching |
| T2.2 | Load built-in plugins from embedded Lua strings |
| T2.3 | Load user plugins from `~/.config/sift/plugins/*.lua` and `SIFT_PLUGINS` env var |
| T2.4 | Implement plugin dispatch: find matching plugin, call execute(), handle result |

## Requirements

### Requirement: Registry precedes plugin loading

T2.1 SHALL complete BEFORE T2.2 SHALL run.

#### Scenario: Registry is ready for registration

- **WHEN** T2.1 creates the plugin registry
- **THEN** T2.2 SHALL register built-in plugins into the registry.

### Requirement: Built-in plugins precede user plugins

T2.2 SHALL complete BEFORE T2.3 SHALL run.

#### Scenario: User plugins override built-ins

- **WHEN** T2.2 registers built-in plugins at priority -1000
- **THEN** T2.3 SHALL register user plugins at their declared priority, allowing override.

### Requirement: Plugin loading precedes dispatch

T2.3 SHALL complete BEFORE T2.4 SHALL run.

#### Scenario: All plugins are available for dispatch

- **WHEN** T2.3 finishes loading user plugins
- **THEN** T2.4 SHALL dispatch commands to the best matching plugin.

### Requirement: Longest-prefix matching

ALWAYS T2.1 SHALL resolve plugins by longest-prefix match, then highest priority.

#### Scenario: Longer prefix wins

- **WHEN** "docker ps" and "docker" are both registered
- **THEN** T2.1 SHALL match "docker ps" for command "docker ps".

#### Scenario: Higher priority overrides

- **WHEN** "cat" is registered with priority -1000 and priority 100
- **THEN** T2.1 SHALL select the priority 100 plugin.

### Requirement: User plugins override built-ins

ALWAYS T2.3 SHALL give user plugins higher priority than built-ins at the same declared priority.

#### Scenario: User cat.lua overrides built-in cat.lua

- **WHEN** both built-in cat.lua and user cat.lua declare priority 0
- **THEN** the user plugin SHALL be selected.

### Requirement: Plugin returns table with name, priority, execute

ALWAYS T2.4 SHALL call the `execute` function on the matched plugin's returned table.

#### Scenario: Plugin executes

- **WHEN** T2.4 finds a matching plugin
- **THEN** T2.4 SHALL call `plugin.execute(ctx, args, stdin)` and handle the result.
