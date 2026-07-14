# sift.* API — Modified Signatures

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add ctx as first argument to all sift.* functions |
| T2.2 | Implement sift.log.nudge(ctx, msg) |
| T3.1 | Implement sift.store(ctx, content, slug) |
| T2.4 | Automatic nudge on sift.exec() non-zero exit |

## ADDED Requirements

### Requirement: sift.log.nudge accumulates messages

ALWAYS T2.2 SHALL accept a message string and accumulate it in the nudge buffer for the current plugin execution.

#### Scenario: Nudge added during plugin execution

- **WHEN** a plugin calls `sift.log.nudge(ctx, "json compressed")`
- **THEN** T2.2 SHALL append `"json compressed"` to the nudge accumulator.

### Requirement: sift.store writes to disk

ALWAYS T3.1 SHALL write content to `/tmp/sift/<session>/<ts>_<count>_<slug>` and emit a nudge with the path.

#### Scenario: Store with custom slug

- **WHEN** a plugin calls `sift.store(ctx, content, "my-output.json")`
- **THEN** T3.1 SHALL write content to a file named with the slug and emit `[sift] use 'command cat <path>' for my-output.json`.

## MODIFIED Requirements

### Requirement: sift.exec saves on error only

ALWAYS T2.4 SHALL save raw combined output to `/tmp/sift/<session>/<ts>_<count>_<slug>.log` only when `exit_code != 0`. On success (exit_code == 0), raw output SHALL NOT be saved.

#### Scenario: Error output saved and nudged

- **WHEN** T2.4 executes a command that exits with code 1
- **THEN** T2.4 SHALL save raw output to disk and emit `[sift] use 'command cat <path>' for raw output`.

#### Scenario: Success output not saved

- **WHEN** T2.4 executes a command that exits with code 0
- **THEN** T2.4 SHALL NOT save raw output to disk and SHALL NOT emit an error nudge.
