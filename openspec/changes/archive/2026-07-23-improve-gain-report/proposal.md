## Why

`sift --gain` shows token reduction stats but the output is ambiguous when no `AI_SESSION` is set — it silently shows all sessions without indicating scope. The report also lacks basic context (date range, session count) and doesn't highlight underperforming plugins, making it harder to identify configuration issues.

## What Changes

- When no `AI_SESSION` is set, show "across N sessions" in the commands line to clarify scope
- Add date range line showing the period covered by the data
- Add total tokens saved alongside the percentage
- Keep all existing behavior unchanged — no new CLI flags, no breaking changes

## Capabilities

### New Capabilities

- `gain-report-formatting`: Enhanced formatting for the `--gain` output — session count, date range, absolute savings. No new flags, no new data sources, just better presentation of existing data.

### Modified Capabilities

*(none — no spec-level behavior changes)*

## Impact

- `sift-core/src/lua/api_reg_io.rs`: `GainReport` struct gets two new optional fields (`session_count`, `first_seen`, `last_seen`). `generate_gain_report` tracks min/max timestamps and unique session count. `format_gain_report` renders the new lines.
- No changes to CLI, Lua API, database schema, or plugin system.
