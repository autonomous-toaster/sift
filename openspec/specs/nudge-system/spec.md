# Nudge System

## Purpose

Provide a unified nudge system that accumulates messages during plugin execution and appends them to output, with automatic nudges on error, unchanged status, and format selection.

## Requirements

### Requirement: Nudge accumulates messages

The system SHALL accumulate nudge messages during plugin execution in a Vec<String> in the SiftLua runtime.

#### Scenario: Multiple nudges from a plugin

- **WHEN** a plugin calls `sift.log.nudge(ctx, "msg1")` then `sift.log.nudge(ctx, "msg2")`
- **THEN** the system SHALL store both messages in the nudge accumulator.

### Requirement: Nudges appended to output at dispatch end

The system SHALL append all accumulated nudges to the plugin's output at the end of dispatch, each prefixed with `[sift] `.

#### Scenario: Nudges appear after plugin output

- **WHEN** a plugin returns `{output="result", status="handled"}` with nudges `["msg1", "msg2"]`
- **THEN** the system SHALL emit `result\n[sift] msg1\n[sift] msg2`.

### Requirement: Auto-nudge on sift.exec() error

The system SHALL emit a nudge when `sift.exec()` returns a non-zero exit code.

#### Scenario: Error output stored and nudged

- **WHEN** the system runs a command that exits with code 1 and the raw output is saved to `/tmp/sift/sess-a/42_cmd.log`
- **THEN** the system SHALL emit `[sift] raw: 'command cat /tmp/sift/sess-a/42_cmd.log'`.

### Requirement: Auto-nudge on unchanged

The system SHALL emit a bypass nudge when a plugin returns `status = "unchanged"`.

#### Scenario: Cached file read nudges bypass

- **WHEN** the system receives `{status="unchanged", message="[sift] foo.rs unchanged"}`
- **THEN** the system SHALL emit `[sift] bypass: 'command cat foo.rs'`.
