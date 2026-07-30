## Why

The `sift gain` report has a "Top bypassed" section that shows cache hit rate for bypassed commands — a metric that is structurally always 0% because bypassed commands don't use the cache. This actively confuses users and provides no actionable information.

Additionally, the section only shows plugin passthroughs and explicit bypasses (`command` prefix). The biggest category of unoptimized commands — those going through `__default__` (no matching plugin, 1470 calls) — is completely invisible.

## What Changes

1. Fix conversation recording for passthrough entries to store the actual plugin name (not hardcoded `"command"`) and the actual command name (not the first word `"command"`)
2. Replace cache hit rate with bypass ratio (bypassed / total calls) and bypass reason (explicit vs plugin passthrough) in the "Top bypassed" section
3. Add a new "Commands without plugins" section showing `__default__` commands aggregated by command name

## Capabilities

### New Capabilities
- `bypass-reason-display`: Show bypass ratio and reason for top bypassed commands instead of meaningless cache hit rate

### Modified Capabilities
- `gain-report`: Existing gain report rendering changes — the "Top bypassed" section format changes, and a new section appears for `__default__` commands

## Impact

- `sift-core/src/lua/api.rs`: `handle_passthrough_status` signature and recording logic
- `sift-core/src/lua/api_reg_io.rs`: `BypassEntry` struct, aggregation logic, rendering
- `sift-core/src/session.rs`: No schema changes (existing fields repurposed)
