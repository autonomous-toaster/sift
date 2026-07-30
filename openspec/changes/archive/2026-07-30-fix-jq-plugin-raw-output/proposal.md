## Why

The jq plugin's `-r` (raw output) path assumes the query result is always a JSON array. When a filter produces a single value (e.g., `.[] | select(.name == "x") | .id`), `sift.json.decode()` returns a Lua scalar (number/string/boolean), not a table. The `ipairs()` loop produces nothing, and the plugin returns empty output with exit code 0 — no error, no hint, just silence.

The agent sees a successful command with zero output, retries with `command jq`, and never uses the jq plugin again. This was observed in a real session: 3 successful jq plugin calls, then 1 silent failure on `-r`, then 10 consecutive `command jq` bypasses.

## What Changes

Fix the `-r` output path in `plugins/jq.lua` to handle both array results (multiple values) and scalar results (single value). Also handle the empty result case (no matches) gracefully.

## Capabilities

### Modified Capabilities
- `jq-plugin`: Fix `-r` raw output handling to support single-value and empty results

## Impact

- `plugins/jq.lua`: ~5 lines changed in the `-r` output path
