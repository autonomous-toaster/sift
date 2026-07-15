# Plugin Optionalization

## Purpose

Split plugins into core (embedded) and optional (filesystem-loaded from `plugins/`), with multi-pattern support.

## Requirements

### Requirement: plugins/ directory scanned at startup

The system SHALL scan the `plugins/` directory at startup and load any `.lua` files as plugins.

### Requirement: plugins/ scanned before user config

The system SHALL scan `plugins/` BEFORE user plugin directories (`~/.config/sift/plugins/`, `$SIFT_PLUGINS`) are scanned.

#### Scenario: Plugin from plugins/ is loaded

- **WHEN** the system starts and finds `plugins/cat.lua`
- **THEN** the system SHALL load it as a user plugin at default priority.

### Requirement: Pattern accepts array of strings

The system SHALL match a plugin if any string in its pattern array matches the command candidate.

#### Scenario: Multi-pattern rtk plugin matches docker

- **WHEN** the system dispatches `docker ps` and a plugin has `pattern = {"docker", "podman"}`
- **THEN** the system SHALL select that plugin.

### Requirement: git_status pattern narrowed to git status

The system SHALL change git_status.lua pattern from `"git"` to `"git status"` so that `git diff` falls through to other plugins (e.g., rtk).

#### Scenario: git status matched, git diff falls through

- **WHEN** a user runs `git status`
- **THEN** the system SHALL match git_status.lua (pattern `"git status"`).

- **WHEN** a user runs `git diff`
- **THEN** the system SHALL NOT match git_status.lua, allowing fallthrough to the next matching plugin.
