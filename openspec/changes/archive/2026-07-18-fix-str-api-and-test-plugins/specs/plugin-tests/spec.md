# Plugin Tests

## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Create `tests_plugins.rs` with smoke test loading all plugins |
| T2.2 | Add per-plugin execution tests for sift-read, cat, head, tail, sed |
| T2.3 | Run `cargo test` to verify all tests pass |

## ADDED Requirements

### Requirement: Tests load actual .lua files from disk

The system SHALL load each `.lua` plugin file from the `plugins/` directory via `std::fs::read_to_string` and register it with the Lua runtime. ALWAYS T2.1 SHALL verify that every `sift.*` sub-table and function is callable from the loaded plugin.

#### Scenario: Smoke test loads all plugins

- **WHEN** T2.1 loads each `.lua` file from `plugins/` and registers it
- **THEN** the system SHALL verify `sift.str`, `sift.fs`, `sift.cache`, `sift.hash`, `sift.json`, `sift.diff`, `sift.env`, `sift.log`, `sift.nudge`, `sift.meta` are all non-nil

### Requirement: Per-plugin execution tests

The system SHALL test each plugin with fixture files to verify correct output. ALWAYS T2.2 SHALL create a temp file, dispatch the plugin, and assert on the output content.

#### Scenario: sift-read executes with fixture file

- **WHEN** T2.2 loads `sift-read.lua` and dispatches with a fixture file path
- **THEN** the system SHALL return the file content

#### Scenario: cat executes with fixture file

- **WHEN** T2.2 loads `cat.lua` and dispatches `cat fixture.txt`
- **THEN** the system SHALL return the file content

#### Scenario: head executes with range

- **WHEN** T2.2 loads `head.lua` and dispatches `head -n 2 fixture.txt`
- **THEN** the system SHALL return the first 2 lines

#### Scenario: tail executes with range

- **WHEN** T2.2 loads `tail.lua` and dispatches `tail -n 2 fixture.txt`
- **THEN** the system SHALL return the last 2 lines

#### Scenario: sed executes with range

- **WHEN** T2.2 loads `sed.lua` and dispatches `sed -n '2,4p' fixture.txt`
- **THEN** the system SHALL return lines 2-4