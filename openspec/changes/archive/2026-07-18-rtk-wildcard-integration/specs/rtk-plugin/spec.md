## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Change rtk.lua pattern from hardcoded list to `"*"` |
| T2.2 | Simplify rtk.lua execute logic: try `rtk <command>`, passthrough on non-zero exit |

## MODIFIED Requirements

### Requirement: rtk uses wildcard pattern

T2.1 SHALL complete BEFORE T2.2 SHALL run.

#### Scenario: Wildcard pattern set
- **WHEN** T2.2 runs
- **THEN** T2.1 SHALL have changed rtk.lua's pattern to `"*"`.

### Requirement: rtk falls through on failure

ALWAYS T2.2 SHALL return `{ status = "passthrough" }` on non-zero exit from `sift.exec()`.

#### Scenario: rtk handles the command
- **WHEN** T2.2 runs `rtk docker ps` via `sift.exec()` and exit code is 0
- **THEN** T2.2 SHALL return rtk's output.

#### Scenario: rtk does not handle the command
- **WHEN** T2.2 runs `rtk unknown-cmd` via `sift.exec()` and exit code is non-zero
- **THEN** T2.2 SHALL return `{ status = "passthrough" }`.
