# TOON Encoding

## Task Reference

| Task ID | Description |
|---------|-------------|
| T7.1 | Add `toon-format` dependency |
| T7.2 | Implement `sift.toon.encode(val)` — Lua table to TOON string |
| T7.3 | Implement `sift.toon.decode(str)` — TOON string to Lua table |

## Requirements

### Requirement: Dependency precedes API

T7.1 SHALL complete BEFORE T7.2 SHALL run.

#### Scenario: toon-format is available

- **WHEN** T7.1 adds the dependency
- **THEN** T7.2 SHALL use `toon-format` for encoding.

### Requirement: sift.toon.encode converts Lua table to TOON

ALWAYS T7.2 SHALL convert a Lua table to a TOON string via `serde_json::Value` intermediate.

#### Scenario: Table encoded as TOON

- **WHEN** a plugin calls `sift.toon.encode({users={{id=1, name="Alice"}}})`
- **THEN** T7.2 SHALL return a TOON string like `users[1]{id,name}: 1,Alice`.

### Requirement: sift.toon.decode converts TOON to Lua table

ALWAYS T7.3 SHALL convert a TOON string to a Lua table.

#### Scenario: TOON decoded to table

- **WHEN** a plugin calls `sift.toon.decode("users[1]{id,name}: 1,Alice")`
- **THEN** T7.3 SHALL return a Lua table `{users={{id=1, name="Alice"}}}`.
