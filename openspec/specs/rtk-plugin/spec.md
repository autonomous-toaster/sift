# Rtk Plugin

## Purpose

Provide a built-in rtk.lua plugin that delegates matching commands (docker, podman, kubectl, etc.) to the rtk binary.

## Requirements

### Requirement: rtk uses wildcard pattern

The rtk.lua plugin SHALL use pattern `"*"` (wildcard) instead of a hardcoded list of command names.

#### Scenario: Wildcard pattern set

- **WHEN** rtk.lua is loaded
- **THEN** its pattern SHALL be `"*"`

### Requirement: rtk falls through on failure

The rtk plugin SHALL attempt to execute `rtk <command>` via `sift.exec()`. On non-zero exit code, it SHALL return `{ status = "passthrough" }` to allow the next plugin to handle the command.

#### Scenario: rtk handles the command

- **WHEN** `rtk docker ps` runs via `sift.exec()` and exit code is 0
- **THEN** the plugin SHALL return rtk's output

#### Scenario: rtk does not handle the command

- **WHEN** `rtk unknown-cmd` runs via `sift.exec()` and exit code is non-zero
- **THEN** the plugin SHALL return `{ status = "passthrough" }`
