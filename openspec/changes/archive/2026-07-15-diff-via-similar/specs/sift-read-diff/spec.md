## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Wire up diff in sift-read.lua: on cache miss, look up old hash, load old content, compute diff, emit if useful |

## MODIFIED Requirements

### Requirement: sift-read emits diff on cache miss

T3.1 SHALL complete AFTER T1.1 AND T2.1 SHALL complete.

#### Scenario: Diff emitted when file changes
- **WHEN** T3.1 reads a file whose content hash differs from the cached hash
- **THEN** T3.1 SHALL compute a unified diff and return it if the diff is significantly smaller than the full content.

#### Scenario: Full content returned when diff is too large
- **WHEN** T3.1 reads a file whose content hash differs and the diff is larger than 90% of the new content
- **THEN** T3.1 SHALL return the full content instead of the diff.
