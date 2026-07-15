## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Update `sift.exec()` Lua binding to accept optional `{ transform = fn }` parameter |

## ADDED Requirements

### Requirement: sift.exec accepts transform callback

T2.1 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: Transform applied to each chunk
- **WHEN** T2.1 runs `echo hello` with `{ transform = function(c) return string.upper(c) end }`
- **THEN** T2.1 SHALL write `"HELLO\n"` to stdout and return `"HELLO\n"`.

### Requirement: Transform is optional

ALWAYS T2.1 SHALL stream raw output when no transform is provided.

#### Scenario: No transform, raw streaming
- **WHEN** T2.1 runs `echo hello` without a transform option
- **THEN** T2.1 SHALL write `"hello\n"` to stdout and return `("hello\n", "", 0)`.
