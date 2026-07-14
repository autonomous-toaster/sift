# Plugin Optionalization — Core/Optional Split and Multi-Pattern

## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Move cat.lua and git_status.lua from `sift/plugins/` to top-level `plugins/` |
| T4.2 | Add `plugins/` directory to plugin discovery scan paths in main.rs |
| T4.3 | Implement multi-pattern support: `pattern` accepts `string \| string[]` |
| T4.4 | Change git_status.lua pattern from `"git"` to `"git status"` |

## ADDED Requirements

### Requirement: plugins/ directory scanned at startup

ALWAYS T4.2 SHALL scan the `plugins/` directory at startup and load any `.lua` files as plugins.

### Requirement: plugins/ scanned before user config

T4.2 SHALL complete BEFORE user plugin directories (`~/.config/sift/plugins/`, `$SIFT_PLUGINS`) are scanned.

#### Scenario: Plugin from plugins/ is loaded

- **WHEN** T4.2 starts and finds `plugins/cat.lua`
- **THEN** T4.2 SHALL load it as a user plugin at default priority.

### Requirement: Pattern accepts array of strings

ALWAYS T4.3 SHALL match a plugin if any string in its pattern array matches the command candidate.

#### Scenario: Multi-pattern rtk plugin matches docker

- **WHEN** T4.3 dispatches `docker ps` and a plugin has `pattern = {"docker", "podman"}`
- **THEN** T4.3 SHALL select that plugin.

### Requirement: git_status pattern narrowed to git status

ALWAYS T4.4 SHALL change git_status.lua pattern from `"git"` to `"git status"` so that `git diff` falls through to other plugins (e.g., rtk).

#### Scenario: git status matched, git diff falls through

- **WHEN** a user runs `git status`
- **THEN** T4.4 SHALL match git_status.lua (pattern `"git status"`).

- **WHEN** a user runs `git diff`
- **THEN** T4.4 SHALL NOT match git_status.lua, allowing fallthrough to the next matching plugin.
