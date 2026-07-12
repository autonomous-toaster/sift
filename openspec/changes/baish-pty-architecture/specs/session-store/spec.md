# Session Store

## Task Reference

| Task ID | Description |
|---------|-------------|
| T6.1 | Move session store to ~/.baish/sessions.db |
| T6.2 | Update PluginContext with session_id |

## Requirements

### Requirement: Session store path precedes context update

T6.1 SHALL complete BEFORE T6.2 SHALL run.

#### Scenario: DB path is known before context

- **WHEN** T6.1 creates the database at ~/.baish/sessions.db
- **THEN** T6.2 SHALL include the session_id in PluginContext.

### Requirement: Single DB for all sessions

ALWAYS T6.1 SHALL use a single database file at ~/.baish/sessions.db.

#### Scenario: Multiple sessions share DB

- **WHEN** two sessions run concurrently with different AI_SESSION values
- **THEN** T6.1 SHALL use the same database file for both.

### Requirement: Context provides cache access

ALWAYS T6.2 SHALL provide cache access through PluginContext.

#### Scenario: Plugin reads cache via context

- **WHEN** a plugin calls cache_get on PluginContext
- **THEN** T6.2 SHALL return the cached entry for the current session.
