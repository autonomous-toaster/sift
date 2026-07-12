# PTY Management

## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Implement PTY creation with portable-pty |
| T2.2 | Implement PTY read loop with line splitting |
| T2.3 | Implement signal forwarding |

## Requirements

### Requirement: PTY creation precedes read loop

T2.1 SHALL complete BEFORE T2.2 SHALL run.

#### Scenario: PTY is ready for reading

- **WHEN** T2.1 creates the PTY and spawns real bash
- **THEN** T2.2 SHALL read from the PTY master.

### Requirement: Read loop precedes signal handling

T2.2 SHALL complete BEFORE T2.3 SHALL run.

#### Scenario: Signals are forwarded to running process

- **WHEN** T2.2 is reading from the PTY
- **THEN** T2.3 SHALL forward SIGINT to the child process group.

### Requirement: PTY streams output

ALWAYS T2.2 SHALL write PTY output to stdout as it arrives.

#### Scenario: Output is not buffered

- **WHEN** real bash writes to the PTY
- **THEN** T2.2 SHALL write the output to stdout within 100ms.

### Requirement: Signals are forwarded

ALWAYS T2.3 SHALL forward SIGINT and SIGTERM to the child process group.

#### Scenario: SIGINT reaches child

- **WHEN** baish receives SIGINT
- **THEN** T2.3 SHALL send SIGINT to the child process group.
