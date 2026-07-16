## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `#diff > 0` check to diff usefulness gate in sift-read.lua |

## MODIFIED Requirements

### Requirement: Diff gate requires non-empty diff

T1.1 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Empty diff falls through to content
- **WHEN** T1.1 reads a file whose content hash differs from cached hash but the content is identical
- **THEN** T1.1 SHALL return the file content, not an empty string.
