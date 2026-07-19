# Plugin Tests

## Purpose

Define smoke tests and per-plugin execution tests that load actual `.lua` files from disk to verify `sift.*` API contract correctness.

## Requirements

### Requirement: Tests load actual .lua files from disk

The system SHALL load each `.lua` plugin file from the `plugins/` directory via `std::fs::read_to_string` and register it with the Lua runtime to verify that every `sift.*` sub-table and function is callable from the loaded plugin.

#### Scenario: Smoke test loads all plugins

- **WHEN** tests load each `.lua` file from `plugins/` and register it
- **THEN** the system SHALL verify `sift.str`, `sift.fs`, `sift.cache`, `sift.hash`, `sift.json`, `sift.diff`, `sift.env`, `sift.log`, `sift.nudge`, `sift.meta` are all non-nil

### Requirement: Per-plugin execution tests

The system SHALL test each plugin with fixture files to verify correct output. Tests SHALL create a temp file, dispatch the plugin, and assert on the output content.

#### Scenario: sift-read executes with fixture file

- **WHEN** tests load `sift-read.lua` and dispatch with a fixture file path
- **THEN** the system SHALL return the file content

#### Scenario: cat executes with fixture file

- **WHEN** tests load `cat.lua` and dispatch `cat fixture.txt`
- **THEN** the system SHALL return the file content

#### Scenario: head executes with range

- **WHEN** tests load `head.lua` and dispatch `head -n 2 fixture.txt`
- **THEN** the system SHALL return the first 2 lines

#### Scenario: tail executes with range

- **WHEN** tests load `tail.lua` and dispatch `tail -n 2 fixture.txt`
- **THEN** the system SHALL return the last 2 lines

#### Scenario: sed executes with range

- **WHEN** tests load `sed.lua` and dispatch `sed -n '2,4p' fixture.txt`
- **THEN** the system SHALL return lines 2-4