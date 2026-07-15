# Plugin System — Multi-Pattern and plugins/ Directory

## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.2 | Add `plugins/` directory to plugin discovery scan paths |
| T4.3 | Implement multi-pattern support: pattern accepts string \| string[] |

## ADDED Requirements

### Requirement: Pattern field accepts string or string[]

ALWAYS T4.3 SHALL accept `pattern` as either a string or an array of strings in the plugin registration table.

#### Scenario: Pattern as single string

- **WHEN** a plugin returns `{name="cat", pattern="cat", ...}`
- **THEN** T4.3 SHALL match it against command candidates as before.

#### Scenario: Pattern as array

- **WHEN** a plugin returns `{name="rtk", pattern={"docker", "podman"}, ...}`
- **THEN** T4.3 SHALL match it if any string in the array matches the command candidate.

### Requirement: plugins/ directory scanned at startup

ALWAYS T4.2 SHALL scan `plugins/` directory for `.lua` files and load them as plugins.

### Requirement: plugins/ scanned before user plugins

T4.2 SHALL complete BEFORE user plugin directories are scanned.

#### Scenario: Plugin from plugins/ directory

- **WHEN** T4.2 starts and finds `plugins/cat.lua`
- **THEN** T4.2 SHALL load it as a user plugin.

## REMOVED Requirements

### Requirement: All plugins embedded in binary

ALWAYS (removed) T4.1 — replaced by core/optional split. Only bash.lua, command.lua, reset.lua SHALL remain embedded. Optional plugins SHALL be loaded from `plugins/` directory.
