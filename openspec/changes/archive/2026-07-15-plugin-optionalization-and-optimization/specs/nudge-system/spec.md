# Nudge System — Explicit and Automatic Agent Messages

## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Implement `sift.log.nudge(ctx, msg)` — accumulate messages during plugin execution |
| T2.2 | Append nudges to plugin output at end of dispatch |
| T2.3 | Implement automatic nudge on `sift.exec()` non-zero exit |
| T2.4 | Implement automatic nudge on plugin returning `status = "unchanged"` |

## ADDED Requirements

### Requirement: Nudge accumulates messages

ALWAYS T2.1 SHALL accumulate nudge messages during plugin execution in a Vec<String> in the SiftLua runtime.

#### Scenario: Multiple nudges from a plugin

- **WHEN** a plugin calls `sift.log.nudge(ctx, "msg1")` then `sift.log.nudge(ctx, "msg2")`
- **THEN** T2.1 SHALL store both messages in the nudge accumulator.

### Requirement: Nudges appended to output at dispatch end

ALWAYS T2.2 SHALL append all accumulated nudges to the plugin's output at the end of dispatch, each prefixed with `[sift] `.

#### Scenario: Nudges appear after plugin output

- **WHEN** a plugin returns `{output="result", status="handled"}` with nudges `["msg1", "msg2"]`
- **THEN** T2.2 SHALL emit `result\n[sift] msg1\n[sift] msg2`.

### Requirement: Auto-nudge on sift.exec() error

ALWAYS T2.3 SHALL emit a nudge when `sift.exec()` returns a non-zero exit code.

#### Scenario: Error output stored and nudged

- **WHEN** T2.3 runs a command that exits with code 1 and the raw output is saved to `/tmp/sift/sess-a/42_cmd.log`
- **THEN** T2.3 SHALL emit `[sift] use 'command cat /tmp/sift/sess-a/42_cmd.log' for raw output`.

### Requirement: Auto-nudge on unchanged

ALWAYS T2.4 SHALL emit a bypass nudge when a plugin returns `status = "unchanged"`.

#### Scenario: Cached file read nudges bypass

- **WHEN** T2.4 receives `{status="unchanged", message="[sift] foo.rs unchanged"}`
- **THEN** T2.4 SHALL emit `[sift] use 'command cat /path/to/foo.rs' for unfiltered content`.
