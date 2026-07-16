## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Create `plugins/head.lua` with range detection |

## ADDED Requirements

### Requirement: head plugin intercepts range reads

T2.1 SHALL complete BEFORE T4.1 SHALL run.

#### Scenario: Head with -n flag
- **WHEN** T2.1 runs `head -n 5 file.txt`
- **THEN** T2.1 SHALL extract range [1,5] and path, read file, check cache, return content or "unchanged".

#### Scenario: Head with -c flag passthrough
- **WHEN** T2.1 runs `head -c 100 file.txt`
- **THEN** T2.1 SHALL return `{ status = "passthrough" }`.
