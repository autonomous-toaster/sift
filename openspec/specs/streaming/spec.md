## Purpose

Remove the `output` field from the bash plugin's return value to eliminate output duplication, since output is already streamed by `sift.exec()` writing to real stdout.

## Requirements

### Requirement: bash.lua plugin

The bash.lua plugin SHALL remove the `output` field from its return value: `{ status = "handled", exit_code = 0 }`. Output is already streamed by `sift.exec()` -> `exec_command()` writing to real stdout. The `dispatch()` function already handles missing `output` via `result.get("output").unwrap_or_default()`. Other plugins (sift-read, cat, sed, head, tail) SHALL continue returning `output` as before.

#### Scenario: No duplicate output

- **WHEN** `bash("echo hello")` is called
- **THEN** it SHALL output "hello" once (not twice)

#### Scenario: Pipeline not broken

- **WHEN** `bash("wc -l < Justfile")` is called
- **THEN** it SHALL output line count once (not duplicated)

#### Scenario: Other plugins unaffected

- **WHEN** `sift-read /path` is called
- **THEN** it SHALL still return content correctly