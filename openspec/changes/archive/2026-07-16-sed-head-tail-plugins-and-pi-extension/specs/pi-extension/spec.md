## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Create `integrations/pi/sift.ts` — override read, intercept bash, propagate AI_SESSION, handle compaction |

## ADDED Requirements

### Requirement: Extension overrides read tool

T4.1 SHALL complete AFTER T1.1, T2.1, T3.1 SHALL complete.

#### Scenario: Read calls sift-read
- **WHEN** T4.1 handles a `read` call with path, offset, limit
- **THEN** T4.1 SHALL execute `sift -c "sift-read <path> <offset> <limit>"` with `AI_SESSION` in env.

### Requirement: Extension intercepts bash tool

ALWAYS T4.1 SHALL wrap every bash command with `sift -c "<command>"`.

#### Scenario: Bash wrapped with sift
- **WHEN** T4.1 handles a `bash` call with command `"cat file.txt"`
- **THEN** T4.1 SHALL execute `sift -c "cat file.txt"` with `AI_SESSION` in env.

### Requirement: Extension propagates AI_SESSION

ALWAYS T4.1 SHALL pass `AI_SESSION` in the environment of every sift execution, merged with `process.env`.

### Requirement: Extension resets cache on compaction

ALWAYS T4.1 SHALL execute `sift -c "reset"` on `session_compact`, `session_tree`, `session_fork`, `session_switch`, and `session_shutdown` events.
