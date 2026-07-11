# Startup and .bashrc Sourcing

## Task Reference

| Task ID | Description |
|---------|-------------|
| T5.1 | Implement .bashrc sourcing on startup |
| T5.2 | Implement source builtin |

## Requirements

### Requirement: .bashrc sourced at startup

T5.1 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: .bashrc sourced on start

- **WHEN** baish starts AND `~/.bashrc` exists
- **THEN** T5.1 SHALL source it.

### Requirement: Graceful error handling for .bashrc

ALWAYS T5.1 SHALL continue startup when .bashrc has parse errors.

#### Scenario: .bashrc with syntax error

- **WHEN** `~/.bashrc` contains invalid syntax
- **THEN** T5.1 SHALL log the error AND continue.

### Requirement: Source preserves state changes

ALWAYS T5.2 SHALL preserve all state changes from sourced files.

#### Scenario: Source changes cwd and env

- **WHEN** `source setup.sh` is executed AND setup.sh contains `cd /tmp` and `export FOO=bar`
- **THEN** session.cwd SHALL be `/tmp` AND session.env SHALL contain `FOO=bar`.
