## Purpose

Prevent the agent from seeing the same unchanged response repeatedly without escalation by tracking recent unchanged commands and appending a stronger hint after 3+ repeats within a 10-second window.

## Requirements

### Requirement: Recent unchanged tracking in SiftLua

The system SHALL track recent unchanged command+message pairs with timestamps to detect repeated identical responses.

- Add a `Mutex<Vec<(String, u128)>>` field to `SiftLua` for tracking recent command+status pairs with timestamps
- In `dispatch()`, when status is "unchanged", track the command+message key with current timestamp
- Prune entries older than 10 seconds before each check
- If the same key appears 3+ times within the 10-second window, append a stronger hint on a new line
- Keep a sliding window of the last 10 entries

#### Scenario: Repeated unchanged command gets stronger hint

- **WHEN** the same cached command runs 3+ times within 10 seconds
- **THEN** a stronger hint SHALL be appended

#### Scenario: Different commands don't interfere

- **WHEN** different commands return unchanged status
- **THEN** their counters SHALL NOT interfere with each other

#### Scenario: Old entries are pruned

- **WHEN** entries are older than 10 seconds
- **THEN** they SHALL be pruned before the next check