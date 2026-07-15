## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Rewrite `exec_command()` to spawn process, read chunks, write to stdout/stderr, collect for return |

## ADDED Requirements

### Requirement: exec_command streams to stdout

ALWAYS T1.1 SHALL write each stdout chunk to the real stdout as it arrives from the child process.

#### Scenario: Long-running command shows progress
- **WHEN** T1.1 runs `sleep 1 && echo done`
- **THEN** T1.1 SHALL write `"done\n"` to stdout after 1 second, not after the process exits.

### Requirement: exec_command streams to stderr

ALWAYS T1.1 SHALL write each stderr chunk to the real stderr as it arrives.

#### Scenario: Stderr appears in real-time
- **WHEN** T1.1 runs `echo err >&2; sleep 1`
- **THEN** T1.1 SHALL write `"err\n"` to stderr immediately, not after the process exits.

### Requirement: exec_command returns full output

ALWAYS T1.1 SHALL return the complete stdout and stderr as strings, identical to the current behavior.

#### Scenario: Return value matches streamed output
- **WHEN** T1.1 runs `echo hello`
- **THEN** T1.1 SHALL return `("hello\n", "", 0)` and the same `"hello\n"` SHALL have been written to stdout.
