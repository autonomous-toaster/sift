# Agent Mode and Human Mode

## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Implement agent mode (-c command) |
| T4.2 | Implement human mode (REPL with TUI) |
| T4.3 | Implement PS1 prompt |

## Requirements

### Requirement: Agent mode precedes human mode

T4.1 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: Agent mode provides execution core

- **WHEN** T4.1 implements command execution with PTY
- **THEN** T4.2 SHALL reuse the same execution pipeline.

### Requirement: Agent mode exits with child code

ALWAYS T4.1 SHALL exit with the child process exit code.

#### Scenario: Exit code propagates

- **WHEN** T4.1 executes a command that exits with code 42
- **THEN** baish SHALL exit with code 42.

### Requirement: Human mode shows TUI

ALWAYS T4.2 SHALL display a TUI with agent view and raw output panes when stdin is a TTY.

#### Scenario: TUI starts on TTY

- **WHEN** baish starts without -c AND stdin is a TTY
- **THEN** T4.2 SHALL start the ratatui TUI.

### Requirement: PS1 shows on human mode

ALWAYS T4.3 SHALL display a prompt in human mode.

#### Scenario: Prompt is shown

- **WHEN** T4.2 is waiting for input
- **THEN** T4.3 SHALL display "baish$ " with hostname and cwd.
