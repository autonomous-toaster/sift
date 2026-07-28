# Rtk Plugin

## Purpose

Provide a built-in rtk.lua plugin that delegates matching commands (ls, git, docker, etc.) to the rtk binary for compact output.

## Requirements

### Requirement: rtk uses specific command patterns

The rtk.lua plugin SHALL use a list of specific command patterns instead of a wildcard `"*"`. The list SHALL match the commands shown in `rtk --help`.

#### Scenario: Pattern list set

- **WHEN** rtk.lua is loaded
- **THEN** its pattern SHALL be a list of specific commands (e.g., `ls`, `git`, `docker`, etc.)

#### Scenario: rtk handles git

- **WHEN** the agent runs `git status`
- **THEN** the rtk plugin SHALL match and delegate to `rtk git status`

#### Scenario: rtk does not handle sed

- **WHEN** the agent runs `sed -i 's/foo/bar/' file`
- **THEN** the rtk plugin SHALL NOT match, and the command SHALL fall through to the shell plugin

### Requirement: rtk falls through on failure

The rtk plugin SHALL attempt to execute `rtk <command>` via `sift.exec()`. On non-zero exit code, it SHALL return `{ status = "passthrough" }` to allow the next plugin to handle the command.

#### Scenario: rtk handles the command

- **WHEN** `rtk docker ps` runs via `sift.exec()` and exit code is 0
- **THEN** the plugin SHALL return rtk's output

#### Scenario: rtk does not handle the command

- **WHEN** `rtk unknown-cmd` runs via `sift.exec()` and exit code is non-zero
- **THEN** the plugin SHALL return `{ status = "passthrough" }`
