## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Add `[nudge] command:` hint to rtk plugin on successful execution |
| T3.2 | Add `[nudge] command:` hint to jq plugin on successful execution |
| T3.3 | Add `[nudge] command:` hint to git plugin on successful execution |

## ADDED Requirements

### Requirement: Transforming plugins SHALL emit `[nudge] command:` hint

T3.1 SHALL complete AFTER T2.1 SHALL complete.

T3.2 SHALL complete AFTER T2.1 SHALL complete.

T3.3 SHALL complete AFTER T2.1 SHALL complete.

#### Scenario: rtk emits command nudge

**WHEN** T3.1 completes and rtk successfully handles a command
**THEN** the output SHALL include `[nudge] command: '<original command>'`

#### Scenario: jq emits command nudge

**WHEN** T3.2 completes and jq successfully filters JSON
**THEN** the output SHALL include `[nudge] command: '<original command>'`

#### Scenario: git emits command nudge

**WHEN** T3.3 completes and git plugin successfully runs
**THEN** the output SHALL include `[nudge] command: '<original command>'`
