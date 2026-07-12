# Session Cache

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Implement SessionStore with SQLite |
| T3.2 | Wire session into REPL loop |

## Requirements

### Requirement: Session store creation precedes REPL

T3.1 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: Session store opens on startup

- **WHEN** T3.1 implements the SessionStore
- **THEN** T3.2 SHALL open the store on startup when `AI_SESSION` is set.

### Requirement: Stateless fallback when no session

ALWAYS T3.2 SHALL run without a session store when `AI_SESSION` is not set.

#### Scenario: No session env var

- **WHEN** `AI_SESSION` is not set
- **THEN** T3.2 SHALL run in stateless mode.

### Requirement: Schema creation order

T3.1 SHALL create the file_cache table BEFORE the conversation_cache table.

#### Scenario: Both tables created

- **WHEN** T3.1 initializes the database
- **THEN** both tables SHALL exist and be queryable.

### Requirement: Command counter increments

ALWAYS T3.2 SHALL increment cmd_count on each command execution.

#### Scenario: Counter increases

- **WHEN** T3.2 dispatches a command
- **THEN** session.cmd_count SHALL increase by 1.
