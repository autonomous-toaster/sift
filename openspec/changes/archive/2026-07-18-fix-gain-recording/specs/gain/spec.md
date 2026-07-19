# Gain

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Change `agent_mode()` to return exit code |
| T1.2 | Call `std::process::exit()` after async context drains |

## MODIFIED Requirements

### Requirement: dispatch records every command execution

The system SHALL record each dispatched command into the `conversation_cache` table. ALWAYS T1.1 SHALL complete BEFORE T1.2 SHALL run. The recording SHALL complete before the process exits.

#### Scenario: Recording completes before exit

- **WHEN** T1.1 runs a command dispatch
- **THEN** the system SHALL ensure the SQLite write completes before the process exits