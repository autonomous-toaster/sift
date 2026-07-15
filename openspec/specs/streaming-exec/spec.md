# Streaming Exec

## Purpose

Provide real-time streaming of command output to stdout/stderr while collecting the full output for the return value.

## Requirements

### Requirement: exec_command streams to stdout

The system SHALL write each stdout chunk to the real stdout as it arrives from the child process.

#### Scenario: Long-running command shows progress
- **WHEN** the system runs `sleep 1 && echo done`
- **THEN** the system SHALL write `"done\n"` to stdout after 1 second, not after the process exits.

### Requirement: exec_command streams to stderr

The system SHALL write each stderr chunk to the real stderr as it arrives.

#### Scenario: Stderr appears in real-time
- **WHEN** the system runs `echo err >&2; sleep 1`
- **THEN** the system SHALL write `"err\n"` to stderr immediately, not after the process exits.

### Requirement: exec_command returns full output

The system SHALL return the complete stdout and stderr as strings, identical to the current behavior.

#### Scenario: Return value matches streamed output
- **WHEN** the system runs `echo hello`
- **THEN** the system SHALL return `("hello\n", "", 0)` and the same `"hello\n"` SHALL have been written to stdout.
