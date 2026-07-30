## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Rename `[sift]` → `[nudge]` in all agent-facing messages across plugins |
| T2.2 | Rename `[sift]` → `[nudge]` in Rust code (nudge collection, unchanged burst) |
| T2.3 | Update tests to expect `[nudge]` prefix |

## MODIFIED Requirements

### Requirement: Agent-facing messages SHALL use `[nudge]` prefix

T2.1 SHALL complete BEFORE T2.3 SHALL start.

T2.2 SHALL complete BEFORE T2.3 SHALL start.

#### Scenario: Cache hit uses nudge prefix

**WHEN** T2.1 completes
**THEN** `[sift] file unchanged (cached)` SHALL become `[nudge] file unchanged (cached)`

#### Scenario: Raw hint uses nudge prefix

**WHEN** T2.1 completes
**THEN** `[sift] raw: 'command cat ...'` SHALL become `[nudge] raw: 'command cat ...'`

#### Scenario: Diff summary uses nudge prefix

**WHEN** T2.2 completes
**THEN** `[sift] N lines changed of M` SHALL become `[nudge] N lines changed of M`

### Requirement: Internal logging SHALL keep `[sift]` prefix

T2.2 SHALL NOT change `[sift]` in log messages (INFO, WARN, ERROR, DEBUG).

#### Scenario: Log messages unchanged

**WHEN** T2.2 completes
**THEN** `sift-core` log messages SHALL still use `[sift] INFO:`, `[sift] WARN:`, etc.
