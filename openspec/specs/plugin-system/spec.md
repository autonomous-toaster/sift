# Plugin System

## Purpose

Define the plugin system with multi-pattern support and filesystem-based plugin loading from the `plugins/` directory.

## Requirements

### Requirement: Pattern field accepts string or string[]

The system SHALL accept `pattern` as either a string or an array of strings in the plugin registration table.

#### Scenario: Pattern as single string

- **WHEN** a plugin returns `{name="cat", pattern="cat", ...}`
- **THEN** the system SHALL match it against command candidates as before.

#### Scenario: Pattern as array

- **WHEN** a plugin returns `{name="rtk", pattern={"docker", "podman"}, ...}`
- **THEN** the system SHALL match it if any string in the array matches the command candidate.

### Requirement: plugins/ directory scanned at startup

The system SHALL scan `plugins/` directory for `.lua` files and load them as plugins.

### Requirement: plugins/ scanned before user plugins

The system SHALL scan `plugins/` BEFORE user plugin directories are scanned.

#### Scenario: Plugin from plugins/ directory

- **WHEN** the system starts and finds `plugins/cat.lua`
- **THEN** the system SHALL load it as a user plugin.
