# Command Rewriting

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.4 | Add rewrite() method to Plugin trait |

## Requirements

### Requirement: Rewrite precedes PTY execution

T3.4 SHALL complete BEFORE T2.2 SHALL run.

#### Scenario: Rewritten command is sent to PTY

- **WHEN** T3.4 implements the rewrite() method
- **THEN** the dispatcher SHALL call rewrite() before sending to the PTY.

### Requirement: Plugin can rewrite commands

ALWAYS T3.4 SHALL provide a default no-op implementation of rewrite().

#### Scenario: Plugin returns rewritten command

- **WHEN** a plugin's rewrite() returns Some("git status --porcelain=v2")
- **THEN** the dispatcher SHALL execute the rewritten command.

#### Scenario: Plugin returns None

- **WHEN** a plugin's rewrite() returns None
- **THEN** the dispatcher SHALL execute the original command.

### Requirement: Rewrite is lossless

ALWAYS T3.4 SHALL NOT modify the command in a way that changes its semantics.

#### Scenario: Rewrite adds flags only

- **WHEN** a plugin rewrites "git status" to "git status --porcelain=v2"
- **THEN** the output SHALL contain the same information as the original command.
