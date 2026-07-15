# sift.json.shortest — Token-Aware JSON Optimization

## Purpose

Provide a token-aware JSON representation selector that picks the most compact format (raw, compacted JSON, or TOON) based on actual token cost including nudge overhead.

## Requirements

### Requirement: shortest selects minimal token representation

The system SHALL measure token cost of each format output plus nudge overhead and select the format with the lowest total cost.

#### Scenario: TOON wins over raw and compacted JSON

- **WHEN** the system receives raw JSON of 10,000 tokens with formats `{json={max_string_len=80}, toon={}}`
- **THEN** the system SHALL return TOON output if its token cost (toon_output + nudge) is lower than compacted JSON and raw.

#### Scenario: Raw wins when nudge overhead exceeds savings

- **WHEN** the system receives raw JSON of 50 tokens
- **THEN** the system SHALL return raw JSON if no format saves more tokens than the nudge overhead.

### Requirement: short format options

The system SHALL accept options for compacted JSON representation.

#### Scenario: Long string truncated

- **WHEN** the system receives `{max_string_len=80}` and a JSON string of 200 characters
- **THEN** the system SHALL truncate the string to 80 characters with a `...` suffix.

#### Scenario: Large array summarized

- **WHEN** the system receives `{max_array_items=5}` and a JSON array of 100 items
- **THEN** the system SHALL show the first 5 items and append `... +95 more`.

### Requirement: shortest stores raw on non-raw selection

The system SHALL store the raw JSON to `/tmp/sift/<session>/<ts>_<cmd_count>_<slug>.json` when a non-raw format is selected.

#### Scenario: Raw stored and nudged

- **WHEN** the system selects TOON over raw
- **THEN** the system SHALL store the raw JSON to disk and emit `[sift] raw: 'command cat <path>'`.

### Requirement: shortest returns raw for invalid JSON

The system SHALL return raw content unchanged when the input is not valid JSON.

#### Scenario: Non-JSON content

- **WHEN** the system receives plain text instead of JSON with `{toon={}}` format
- **THEN** the system SHALL return the plain text unchanged with no nudge.
