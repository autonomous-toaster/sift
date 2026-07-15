# Rtk Plugin

## Purpose

Provide a built-in rtk.lua plugin that delegates matching commands (docker, podman, kubectl, etc.) to the rtk binary.

## Requirements

### Requirement: Rtk plugin delegates matching commands

The system SHALL execute `rtk <original_command>` via `sift.exec()` for any command matching its pattern list.

#### Scenario: Docker ps delegated to rtk

- **WHEN** a user runs `docker ps`
- **THEN** the system SHALL execute `rtk docker ps` and return rtk's compact output.

### Requirement: Rtk plugin does not override specific plugins

The system SHALL NOT intercept commands that have a more specific plugin (longer pattern match). This is guaranteed by the dispatch system's longest-prefix matching, not by logic within rtk.lua.

#### Scenario: git status handled by git_status.lua, not rtk

- **WHEN** a user runs `git status` and git_status.lua has pattern `"git status"` (length 10) while rtk has pattern `"git"` (length 3)
- **THEN** the dispatch SHALL select git_status.lua (longer pattern wins).

#### Scenario: git diff delegated to rtk

- **WHEN** a user runs `git diff` and no plugin has pattern `"git diff"`
- **THEN** the dispatch SHALL select rtk (pattern `"git"` matches the `"git"` candidate).
