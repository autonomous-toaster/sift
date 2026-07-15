# Output Storage — On-Error Save with Auto-Nudge

## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.4 | Change output storage from always-save to on-error-save with auto-nudge |

## MODIFIED Requirements

### Requirement: Raw output saved on error only

ALWAYS T2.4 SHALL save raw combined output to `/tmp/sift/<session>/<ts>_<count>_<slug>.log` only when `exit_code != 0`.

#### Scenario: Non-zero exit triggers save

- **WHEN** T2.4 runs a command that exits with code 1
- **THEN** T2.4 SHALL write raw output to a temp file.

#### Scenario: Zero exit skips save

- **WHEN** T2.4 runs a command that exits with code 0
- **THEN** T2.4 SHALL NOT write any temp file.

### Requirement: Error save emits nudge

ALWAYS T2.4 SHALL emit a nudge `[sift] use 'command cat <path>' for raw output` when saving on error.

#### Scenario: Nudge with path

- **WHEN** T2.4 saves output to `/tmp/sift/sess-a/42_cmd.log`
- **THEN** T2.4 SHALL emit `[sift] use 'command cat /tmp/sift/sess-a/42_cmd.log' for raw output`.

## REMOVED Requirements

### Requirement: Always save raw output

ALWAYS (removed) T2.4 — replaced by on-error save. Raw output SHALL be saved only when `exit_code != 0`. On success, output SHALL be discarded.
