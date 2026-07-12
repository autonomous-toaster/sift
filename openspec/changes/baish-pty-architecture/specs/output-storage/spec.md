# Full Output Storage

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.5 | Implement full output storage to temp files |
| T3.6 | Implement temp file cleanup |

## Requirements

### Requirement: Output storage precedes cleanup

T3.5 SHALL complete BEFORE T3.6 SHALL run.

#### Scenario: Cleanup depends on storage

- **WHEN** T3.5 writes output to temp files
- **THEN** T3.6 SHALL clean them up on session end.

### Requirement: Full output is stored when truncated

ALWAYS T3.5 SHALL write full raw PTY output to a temp file when the filter truncates or summarizes.

#### Scenario: Truncated output has full version

- **WHEN** a filter emits a truncated or summarized output
- **THEN** T3.5 SHALL save the full output to `/tmp/baish/<session_id>/<timestamp>_<command>.log`.

### Requirement: Agent gets path to full output

ALWAYS T3.5 SHALL include the full output path in the agent-visible output.

#### Scenario: Path is included in output

- **WHEN** T3.5 stores full output to a temp file
- **THEN** the agent output SHALL contain "Full output: /tmp/baish/...".

### Requirement: Temp files are cleaned up

ALWAYS T3.6 SHALL remove the session temp directory when the session ends.

#### Scenario: Cleanup on exit

- **WHEN** baish exits
- **THEN** T3.6 SHALL remove `/tmp/baish/<session_id>/`.

### Requirement: Disk usage is bounded

ALWAYS T3.6 SHALL enforce a configurable max disk usage for temp files.

#### Scenario: Max size exceeded

- **WHEN** temp file total exceeds the configured max
- **THEN** T3.6 SHALL remove the oldest files first.
