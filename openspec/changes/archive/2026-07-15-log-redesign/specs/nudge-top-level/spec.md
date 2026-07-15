## ADDED Requirements

### Requirement: sift.nudge is a top-level function

ALWAYS T2.1 SHALL register `sift.nudge(ctx, msg)` as a standalone function on the `sift` table, independent of `sift.log`.

#### Scenario: Nudge accumulates message
- **WHEN** a plugin calls `sift.nudge(ctx, "bypass: 'command cat foo'")`
- **THEN** T2.1 SHALL append `"bypass: 'command cat foo'"` to the nudge accumulator.

#### Scenario: Multiple nudges
- **WHEN** a plugin calls `sift.nudge(ctx, "msg1")` then `sift.nudge(ctx, "msg2")`
- **THEN** T2.1 SHALL store both messages in the nudge accumulator.

### Requirement: sift.log.nudge is removed

ALWAYS T2.1 SHALL NOT register `sift.log.nudge`. Plugins MUST use `sift.nudge` instead.

#### Scenario: Old nudge path errors
- **WHEN** a plugin calls `sift.log.nudge(ctx, "msg")`
- **THEN** T2.1 SHALL raise a Lua error (nil value).
