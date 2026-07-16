## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Create `plugins/sed.lua` with range detection and passthrough |

## ADDED Requirements

### Requirement: sed plugin intercepts range reads

T1.1 SHALL complete BEFORE T4.1 SHALL run.

#### Scenario: Range print intercepted
- **WHEN** T1.1 runs `sed -n '5,10p' file.txt`
- **THEN** T1.1 SHALL extract range [5,10] and path, read file, check cache, return content or "unchanged".

#### Scenario: Substitution passthrough
- **WHEN** T1.1 runs `sed 's/foo/bar/g' file.txt`
- **THEN** T1.1 SHALL return `{ status = "passthrough" }`.

#### Scenario: No -n flag passthrough
- **WHEN** T1.1 runs `sed '5,10p' file.txt` without `-n`
- **THEN** T1.1 SHALL return `{ status = "passthrough" }`.
