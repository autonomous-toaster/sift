# Fix Gain Tracking for head, tail, sed, openspec plugins

## Problem

The `sift gain` report shows 0% token reduction for `head`, `tail`, `sed`, and `openspec` plugins, even though they all optimize output. This is because these plugins don't set `raw_bytes` in their return tables. The dispatch code falls back to `filtered_bytes` when `raw_bytes` is missing, making the reduction calculation `(raw - filtered) / raw = 0%`.

## Impact

- `head`, `tail`, `sed` read files and return a subset (range read). The gain report should show the savings from returning only N lines instead of the full file.
- `openspec` injects `--json` and converts output via `sift.json.shortest()`. The gain report should show the savings from JSON compression.
- Without accurate gain data, the user can't tell which plugins are actually saving tokens.

## Scope

Four plugins need `raw_bytes` added to their return tables:
- `plugins/head.lua`
- `plugins/tail.lua`
- `plugins/sed.lua`
- `plugins/openspec.lua`

No Rust code changes. No new dependencies. No behavior changes — only gain tracking accuracy.
