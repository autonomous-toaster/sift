## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Change `pattern = "*"` to list of known rtk commands |
| T1.2 | Verify `sed -i 's/foo/bar/' file` no longer matches rtk |
| T1.3 | Verify `git status` still matches rtk |

## ADDED Requirements

### Requirement: rtk plugin SHALL use specific command patterns

T1.1 SHALL complete BEFORE T1.2 SHALL start.

T1.1 SHALL complete BEFORE T1.3 SHALL start.

#### Scenario: rtk handles git

**WHEN** T1.3 runs
**THEN** `git status` SHALL match the rtk plugin.

#### Scenario: rtk does not handle sed

**WHEN** T1.2 runs
**THEN** `sed -i 's/foo/bar/' file` SHALL NOT match the rtk plugin.
