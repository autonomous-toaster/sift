# Output Storage

## Purpose

Change output storage from always-save to on-error-save with auto-nudge.

## Requirements

### Requirement: Raw output saved on error only

The system SHALL save raw combined output to `/tmp/sift/<session>/<ts>_<count>_<slug>.log` only when `exit_code != 0`.

#### Scenario: Non-zero exit triggers save

- **WHEN** the system runs a command that exits with code 1
- **THEN** the system SHALL write raw output to a temp file.

#### Scenario: Zero exit skips save

- **WHEN** the system runs a command that exits with code 0
- **THEN** the system SHALL NOT write any temp file.

### Requirement: Error save emits nudge

The system SHALL emit a nudge `[sift] raw: 'command cat <path>'` when saving on error.

#### Scenario: Nudge with path

- **WHEN** the system saves output to `/tmp/sift/sess-a/42_cmd.log`
- **THEN** the system SHALL emit `[sift] raw: 'command cat /tmp/sift/sess-a/42_cmd.log'`.
