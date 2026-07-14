# Rtk Plugin — Delegate Commands to rtk

## Task Reference

| Task ID | Description |
|---------|-------------|
| T6.1 | Write `plugins/rtk.lua` — delegates docker, podman, kubectl, oc, gh, glab, curl, wget, npm, pnpm, pip, uv to rtk |
| T6.2 | Ensure rtk plugin has lower effective priority than specific plugins (git status, etc.) via pattern specificity |

## ADDED Requirements

### Requirement: Rtk plugin delegates matching commands

ALWAYS T6.1 SHALL execute `rtk <original_command>` via `sift.exec()` for any command matching its pattern list.

#### Scenario: Docker ps delegated to rtk

- **WHEN** a user runs `docker ps`
- **THEN** T6.1 SHALL execute `rtk docker ps` and return rtk's compact output.

### Requirement: Rtk plugin does not override specific plugins

ALWAYS T6.2 SHALL NOT intercept commands that have a more specific plugin (longer pattern match). This is guaranteed by the dispatch system's longest-prefix matching, not by logic within rtk.lua.

#### Scenario: git status handled by git_status.lua, not rtk

- **WHEN** a user runs `git status` and git_status.lua has pattern `"git status"` (length 10) while rtk has pattern `"git"` (length 3)
- **THEN** the dispatch SHALL select git_status.lua (longer pattern wins).

#### Scenario: git diff delegated to rtk

- **WHEN** a user runs `git diff` and no plugin has pattern `"git diff"`
- **THEN** the dispatch SHALL select rtk (pattern `"git"` matches the `"git"` candidate).
