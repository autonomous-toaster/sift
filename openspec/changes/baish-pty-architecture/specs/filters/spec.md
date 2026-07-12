# Output Filters

## Task Reference

| Task ID | Description |
|---------|-------------|
| T5.1 | Implement CatFilter |
| T5.2 | Implement CargoTestFilter |
| T5.3 | Implement GitStatusFilter |

## Requirements

### Requirement: CatFilter precedes other filters

T5.1 SHALL complete BEFORE T5.2 SHALL run.

#### Scenario: CatFilter validates filter pattern

- **WHEN** T5.1 implements the CatFilter
- **THEN** T5.2 SHALL follow the same StreamFilter pattern.

### Requirement: CatFilter deduplicates output

ALWAYS T5.1 SHALL emit "unchanged" marker when output hash matches cache.

#### Scenario: Repeated read emits marker

- **WHEN** T5.1 receives output identical to a previous command
- **THEN** T5.1 SHALL emit "[baish] <file> unchanged since last read".

### Requirement: CargoTestFilter summarizes on success

ALWAYS T5.2 SHALL emit a summary line on successful test run.

#### Scenario: All tests pass

- **WHEN** T5.2 receives cargo test output with exit code 0
- **THEN** T5.2 SHALL emit "✓ N tests passed".

### Requirement: CargoTestFilter shows failures

ALWAYS T5.2 SHALL show failed test names and locations on failure.

#### Scenario: Tests fail

- **WHEN** T5.2 receives cargo test output with exit code non-zero
- **THEN** T5.2 SHALL emit failed test names and panic locations.

### Requirement: GitStatusFilter fingerprints

ALWAYS T5.3 SHALL compute a fingerprint of HEAD + index + worktree.

#### Scenario: Clean tree

- **WHEN** T5.3 receives git status output AND fingerprint matches
- **THEN** T5.3 SHALL emit "working tree clean".
