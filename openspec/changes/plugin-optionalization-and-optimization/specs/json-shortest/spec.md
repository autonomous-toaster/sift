# sift.json.shortest — Token-Aware JSON Optimization

## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Implement `sift.json.shortest(ctx, raw, formats)` in Rust |
| T4.2 | Implement compacted JSON output (max_string_len, max_array_items, max_depth, max_keys) |
| T4.3 | Wire automatic nudge + raw storage when non-raw format wins |
| T4.4 | Add tests for token cost comparison and format selection |

## ADDED Requirements

### Requirement: shortest selects minimal token representation

ALWAYS T4.1 SHALL measure token cost of each format output plus nudge overhead and select the format with the lowest total cost.

#### Scenario: TOON wins over raw and compacted JSON

- **WHEN** T4.1 receives raw JSON of 10,000 tokens with formats `{json={max_string_len=80}, toon={}}`
- **THEN** T4.1 SHALL return TOON output if its token cost (toon_output + nudge) is lower than compacted JSON and raw.

#### Scenario: Raw wins when nudge overhead exceeds savings

- **WHEN** T4.1 receives raw JSON of 50 tokens
- **THEN** T4.1 SHALL return raw JSON if no format saves more tokens than the nudge overhead.

### Requirement: short format options

ALWAYS T4.2 SHALL accept options for compacted JSON representation.

#### Scenario: Long string truncated

- **WHEN** T4.2 receives `{max_string_len=80}` and a JSON string of 200 characters
- **THEN** T4.2 SHALL truncate the string to 80 characters with a `...` suffix.

#### Scenario: Large array summarized

- **WHEN** T4.2 receives `{max_array_items=5}` and a JSON array of 100 items
- **THEN** T4.2 SHALL show the first 5 items and append `... +95 more`.

### Requirement: shortest stores raw on non-raw selection

ALWAYS T4.3 SHALL store the raw JSON to `/tmp/sift/<session>/<ts>_<cmd_count>_<slug>.json` when a non-raw format is selected.

#### Scenario: Raw stored and nudged

- **WHEN** T4.3 selects TOON over raw
- **THEN** T4.3 SHALL store the raw JSON to disk and emit `[sift] use 'command cat <path>' for raw original`.

### Requirement: shortest returns raw for invalid JSON

ALWAYS T4.1 SHALL return raw content unchanged when the input is not valid JSON.

#### Scenario: Non-JSON content

- **WHEN** T4.1 receives plain text instead of JSON with `{toon={}}` format
- **THEN** T4.1 SHALL return the plain text unchanged with no nudge.
