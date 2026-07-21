# Gain

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Wire `record_conversation()` into `dispatch()` after plugin execution |
| T1.2 | Capture raw/filtered bytes in passthrough path |
| T1.3 | Add `sift.gain.report()` Rust function |
| T1.5 | Create `gain.lua` plugin |
| T2.1 | Add `raw_bytes` to cat.lua return |
| T2.2 | Add `raw_bytes` to sift-read.lua return |
| T3.1 | Test gain report with in-memory recording |

## ADDED Requirements

### Requirement: dispatch records every command execution

The system SHALL record each dispatched command into the `conversation_cache` table with `raw_bytes`, `filtered_bytes`, `plugin_name`, and `output_format`. ALWAYS T1.1 SHALL extract `raw_bytes` from the plugin result table (optional, fallback to `filtered_bytes`). ALWAYS T1.1 SHALL compute `filtered_bytes` as the length of the final output string (including nudge text). ALWAYS T1.1 SHALL call `record_conversation()` via `tokio::spawn` to avoid blocking the Lua dispatch.

#### Scenario: Handled plugin records raw and filtered bytes

- **WHEN** T1.1 runs after `dispatch()` finishes a handled plugin execution
- **THEN** the system SHALL record `raw_bytes`, `filtered_bytes`, `plugin_name`, and `output_format` in `conversation_cache`

#### Scenario: Passthrough records bypass

- **WHEN** T1.2 runs after `dispatch()` finishes a passthrough execution
- **THEN** the system SHALL record `plugin_name = "command"` and `output_format = "passthrough"` with `raw_bytes = filtered_bytes`

### Requirement: plugins report raw_bytes

The system SHALL allow plugins to report `raw_bytes` as an optional field in their return table. ALWAYS T2.1, T2.2 SHALL add `raw_bytes` to the return table of file-reading plugins. When absent, the system SHALL fall back to `filtered_bytes` (0% reduction assumed).

#### Scenario: cat.lua reports file size

- **WHEN** T2.1 runs and cat.lua returns command output
- **THEN** the system SHALL include `raw_bytes` equal to the file size on disk

#### Scenario: sift-read.lua reports file size

- **WHEN** T2.2 runs and sift-read.lua returns command output
- **THEN** the system SHALL include `raw_bytes` equal to the file size on disk

### Requirement: sift.gain.report() queries session store

The system SHALL expose `sift.gain.report(args)` as a Rust-registered Lua function. ALWAYS T1.3 SHALL query the `conversation_cache` table and aggregate stats. ALWAYS T1.3 SHALL support flags: `--verbose` (per-command list), `--json` (machine-readable), `--all` (all sessions), `--session <id>` (specific session), `--since <timestamp>` (time filter).

#### Scenario: Default report shows aggregate

- **WHEN** T1.3 runs `sift.gain.report({})` with recorded data
- **THEN** the system SHALL return a formatted string with total commands, raw bytes, filtered bytes, reduction percentage, bypass count, per-plugin breakdown, and session info

#### Scenario: JSON report is machine-readable

- **WHEN** T1.3 runs `sift.gain.report({json = true})`
- **THEN** the system SHALL return a JSON string with structured data

### Requirement: gain.lua dispatches sift gain

The system SHALL dispatch `sift gain` through a Lua plugin. ALWAYS T1.5 SHALL match `"sift gain"` pattern and call `sift.gain.report()`. ALWAYS T1.5 SHALL forward flags (`--verbose`, `--json`, `--all`, `--session`, `--since`) as arguments.

#### Scenario: sift gain without flags

- **WHEN** T1.5 runs `sift gain`
- **THEN** the system SHALL call `sift.gain.report({})` and return the formatted output

#### Scenario: sift gain --json

- **WHEN** T1.5 runs `sift gain --json`
- **THEN** the system SHALL call `sift.gain.report({json = true})` and return JSON output