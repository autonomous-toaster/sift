## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Create `plugins/tail.lua` with range detection |

## ADDED Requirements

### Requirement: tail plugin intercepts range reads

T3.1 SHALL complete BEFORE T4.1 SHALL run.

#### Scenario: Tail with -n flag
- **WHEN** T3.1 runs `tail -n 5 file.txt`
- **THEN** T3.1 SHALL read full file, compute range [total-4, total], check cache, return content or "unchanged".

#### Scenario: Tail with -c flag passthrough
- **WHEN** T3.1 runs `tail -c 100 file.txt`
- **THEN** T3.1 SHALL return `{ status = "passthrough" }`.
