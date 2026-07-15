## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `similar` crate and expose `sift.diff(ctx, old, new)` returning unified diff |

## ADDED Requirements

### Requirement: sift.diff returns unified diff

T1.1 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Diff between two texts
- **WHEN** T1.1 computes diff between `"line1\nline2\n"` and `"line1\nline2 modified\n"`
- **THEN** T1.1 SHALL return a unified diff showing the changed line.
