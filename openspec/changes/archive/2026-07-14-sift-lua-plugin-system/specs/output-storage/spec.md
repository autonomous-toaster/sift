# Output Storage

## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Implement automatic raw output saving in `sift.exec()` |
| T4.2 | Implement format detection (JSON, TOON, text) |
| T4.3 | Implement temp file cleanup on session end |
| T4.4 | Implement configurable max disk usage |

## Requirements

### Requirement: Output storage precedes cleanup

T4.1 SHALL complete BEFORE T4.3 SHALL run.

#### Scenario: Cleanup depends on storage

- **WHEN** T4.1 writes output to temp files
- **THEN** T4.3 SHALL clean them up on session end.

### Requirement: Raw output is always saved

ALWAYS T4.1 SHALL save raw PTY output to a temp file on every `sift.exec()` call.

#### Scenario: Output is saved

- **WHEN** `sift.exec()` runs a command
- **THEN** the raw output SHALL be written to `/tmp/sift/<session_id>/<cmd_count>_<slug>.log`.

### Requirement: Format is detected from content

ALWAYS T4.2 SHALL detect the format by inspecting the first bytes of output.

#### Scenario: JSON detected

- **WHEN** output starts with `{` or `[`
- **THEN** T4.2 SHALL set format to `json`.

#### Scenario: TOON detected

- **WHEN** output matches the TOON header pattern
- **THEN** T4.2 SHALL set format to `toon`.

#### Scenario: Text fallback

- **WHEN** output does not match JSON or TOON patterns
- **THEN** T4.2 SHALL set format to `text`.

### Requirement: Temp files are cleaned up

T4.3 SHALL complete AFTER T4.1.

#### Scenario: Cleanup on exit

- **WHEN** sift exits
- **THEN** T4.3 SHALL remove `/tmp/sift/<session_id>/`.
