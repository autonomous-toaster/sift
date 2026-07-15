# sift.* API

## Purpose

Define the modified sift.* API signatures including ctx-first, nudge, store, and auto-nudge on exec error.

## Requirements

### Requirement: sift.log.nudge accumulates messages

The system SHALL accept a message string and accumulate it in the nudge buffer for the current plugin execution.

#### Scenario: Nudge added during plugin execution

- **WHEN** a plugin calls `sift.log.nudge(ctx, "json compressed")`
- **THEN** the system SHALL append `"json compressed"` to the nudge accumulator.

### Requirement: sift.store writes to disk

The system SHALL write content to `/tmp/sift/<session>/<ts>_<count>_<slug>` and emit a nudge with the path.

#### Scenario: Store with custom slug

- **WHEN** a plugin calls `sift.store(ctx, content, "my-output.json")`
- **THEN** the system SHALL write content to a file named with the slug and emit `[sift] stored: 'command cat <path>'`.

### Requirement: sift.exec saves on error only

The system SHALL save raw combined output to `/tmp/sift/<session>/<ts>_<count>_<slug>.log` only when `exit_code != 0`. On success (exit_code == 0), raw output SHALL NOT be saved.

#### Scenario: Error output saved and nudged

- **WHEN** the system executes a command that exits with code 1
- **THEN** the system SHALL save raw output to disk and emit `[sift] raw: 'command cat <path>'`.

#### Scenario: Success output not saved

- **WHEN** the system executes a command that exits with code 0
- **THEN** the system SHALL NOT save raw output to disk and SHALL NOT emit an error nudge.
