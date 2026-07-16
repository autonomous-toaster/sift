## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Add regression tests for empty diff, range boundary, cross-range unchanged |

## ADDED Requirements

### Requirement: Regression tests cover known bugs

T2.1 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: Range boundary after smaller range
- **WHEN** T2.1 reads lines 1-4 then lines 1-5 of the same file
- **THEN** T2.1 SHALL return content for lines 1-5, not empty.

#### Scenario: Cross-range unchanged detection
- **WHEN** T2.1 reads lines 1-4 then lines 1-4 again
- **THEN** T2.1 SHALL return "unchanged".

#### Scenario: Sub-range after larger range
- **WHEN** T2.1 reads lines 1-10 then lines 3-5
- **THEN** T2.1 SHALL return content for lines 3-5 (sub-range is not satisfied by larger range).
