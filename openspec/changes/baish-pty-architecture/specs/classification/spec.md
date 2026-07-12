# Command Classification and Dispatch

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Implement CommandKind enum and classify() |
| T3.2 | Implement decision engine |
| T3.3 | Implement pre-compute hash logic |
| T3.4 | Add rewrite() method to Plugin trait |
| T3.5 | Implement full output storage to temp files |
| T3.6 | Implement temp file cleanup |

## Requirements

### Requirement: Classification precedes decision engine

T3.1 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: Classifier feeds decision engine

- **WHEN** T3.1 classifies a command as FileRead
- **THEN** T3.2 SHALL check the cache before executing.

### Requirement: Decision engine precedes hash logic

T3.2 SHALL complete BEFORE T3.3 SHALL run.

#### Scenario: Hash logic depends on decision

- **WHEN** T3.2 decides to execute a FileRead command
- **THEN** T3.3 SHALL pre-compute the file hash.

### Requirement: Simple FileRead skips PTY on cache hit

ALWAYS T3.2 SHALL skip PTY execution when a simple FileRead has a cache hit.

#### Scenario: Cached file is not re-read

- **WHEN** T3.2 receives "cat foo.rs" AND the file hash matches the cache
- **THEN** T3.2 SHALL emit the cached marker AND NOT spawn a PTY.

### Requirement: Compound commands always use PTY

ALWAYS T3.2 SHALL execute compound commands through the PTY.

#### Scenario: Compound command goes to PTY

- **WHEN** T3.2 receives "cd /x && cat foo.rs"
- **THEN** T3.2 SHALL send the command to the PTY.
