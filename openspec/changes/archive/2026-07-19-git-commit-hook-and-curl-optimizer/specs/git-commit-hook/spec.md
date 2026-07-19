## ADDED Requirements

### Requirement: Forbid --no-verify on git commit

The `git-commit` plugin SHALL intercept `git commit` commands and detect `-n` or `--no-verify` flags. When detected, it SHALL return a non-zero exit code and emit a nudge explaining that hooks must run. All other git commands SHALL passthrough to the default handler (rtk).

#### Scenario: git commit with -n flag
- **WHEN** agent runs `git commit -m "fix" -n`
- **THEN** plugin returns exit code 1 with empty output
- **AND** plugin emits nudge: "git commit --no-verify (-n) is forbidden: hooks must run"

#### Scenario: git commit with --no-verify flag
- **WHEN** agent runs `git commit --no-verify -m "fix"`
- **THEN** plugin returns exit code 1 with empty output
- **AND** plugin emits nudge

#### Scenario: git commit without flags
- **WHEN** agent runs `git commit -m "fix"`
- **THEN** plugin passthrough to rtk

#### Scenario: non-commit git commands
- **WHEN** agent runs `git status` or `git push`
- **THEN** plugin passthrough to rtk

#### Scenario: -n in commit message value (false positive)
- **WHEN** agent runs `git commit -m "fix -n issue"`
- **THEN** plugin does NOT detect `-n` (it's a value to `-m`, not a flag)
- **AND** plugin passthrough to rtk
