## Why

When the pi agent runs `veriplan check` through sift's bash proxy, the output is human-readable by default. This wastes tokens because:
1. Human format is verbose (explanatory text, line wrapping)
2. No compression (JSON compact or toon)
3. Agent doesn't need the human-friendly formatting

A sift plugin for veriplan ensures machine-readable JSON output and compresses it to the shortest format (compact JSON vs toon), reducing token consumption for the agent.

## What Changes

### New: `plugins/veriplan.lua`
Sift plugin that intercepts `veriplan check` commands, ensures `--json` output, and compresses via `sift.json.shortest()`.

### Modified: `sift-core/src/lua/api_reg_io.rs`
Fix `json_shortest_impl()` nudge overhead calculation — the hardcoded `20` for the nudge message prefix is stale (should be `38` for the current "compressed output. raw: command cat " format).

## Capabilities

### New Capabilities
- `veriplan-plugin`: Sift plugin for veriplan output optimization

### Modified Capabilities
- `json-shortest`: Fix nudge overhead calculation in `json_shortest_impl()`
