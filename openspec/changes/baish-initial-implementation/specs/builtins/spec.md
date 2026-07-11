# Builtins

## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.3 | Implement builtins (cd, export, unset, exit) |
| T5.2 | Implement source builtin |

## Requirements

### Requirement: Builtins implemented before REPL

T2.3 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: Builtins recognized by dispatcher

- **WHEN** T2.3 implements cd, export, unset, exit
- **THEN** the dispatcher SHALL recognize these commands as builtins.

### Requirement: cd updates working directory

ALWAYS T2.3 SHALL update session.cwd when cd is executed.

#### Scenario: cd to path

- **WHEN** `cd /tmp` is executed
- **THEN** session.cwd SHALL be `/tmp`.

#### Scenario: cd home

- **WHEN** `cd` is executed
- **THEN** session.cwd SHALL be `$HOME`.

### Requirement: export updates environment

ALWAYS T2.3 SHALL update session.env when export is executed.

#### Scenario: export variable

- **WHEN** `export FOO=bar` is executed
- **THEN** session.env SHALL contain `FOO=bar`.

### Requirement: Source builtin precedes REPL

T5.2 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: Source preserves state

- **WHEN** `source setup.sh` is executed AND setup.sh contains `cd /tmp` and `export FOO=bar`
- **THEN** session.cwd SHALL be `/tmp` AND session.env SHALL contain `FOO=bar`.
