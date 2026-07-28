## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Add `original_cmd` field to context table in `dispatch()` |
| T2.2 | Verify plugins can access `ctx.original_cmd` |

## MODIFIED Requirements

### Requirement: Plugin context SHALL include `original_cmd`

T2.1 SHALL complete BEFORE T2.2 SHALL start.

T2.1 SHALL complete BEFORE T3.2 SHALL start.

#### Scenario: Plugin accesses original command

**WHEN** T2.2 runs
**THEN** `ctx.original_cmd` SHALL contain the exact command string as typed by the agent.
