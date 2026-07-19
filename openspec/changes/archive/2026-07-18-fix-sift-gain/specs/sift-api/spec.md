# sift.* API

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `--gain` CLI flag |
| T2.1 | Fix `record_conversation` panic |

## MODIFIED Requirements

### Requirement: sift.gain.report() queries session store

The system SHALL register `sift.gain.report` as a function taking a table of flags and returning a formatted report string. ALWAYS T1.1 SHALL add `--gain` as the primary CLI entry point. ALWAYS T2.1 SHALL fix `record_conversation` to handle non-tokio contexts without panicking.

#### Scenario: CLI --gain prints report

- **WHEN** T1.1 runs `sift --gain`
- **THEN** the system SHALL print the gain report to stdout and exit

#### Scenario: record_conversation without tokio runtime

- **WHEN** T2.1 calls `record_conversation` outside a tokio context
- **THEN** the system SHALL spawn a new thread with a dedicated runtime instead of panicking