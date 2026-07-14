# jq Queries

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.11 | Implement `sift.jq.query(data, filter)` — execute jq filter on JSON data |

## Requirements

### Requirement: Dependency precedes API

T3.11 SHALL add the `jaq` dependency before implementing the query function.

#### Scenario: jaq is available

- **WHEN** T3.11 adds the `jaq` dependency
- **THEN** T3.11 SHALL use `jaq` for filter execution.

### Requirement: sift.jq.query supports full jq syntax

ALWAYS T3.11 SHALL support the full jq filter syntax as implemented by the `jaq` crate.

#### Scenario: Simple field access

- **WHEN** a plugin calls `sift.jq.query('{"users":[{"name":"Alice"}]}', '.users[].name')`
- **THEN** T3.11 SHALL return `"Alice"`.

#### Scenario: Complex filter with select

- **WHEN** a plugin calls `sift.jq.query(data, 'map(select(.status == "FAILED")) | .[].name')`
- **THEN** T3.11 SHALL return the filtered result as a JSON string.

### Requirement: sift.jq.query accepts both string and table

ALWAYS T3.11 SHALL accept `data` as either a JSON string or a Lua table.

#### Scenario: Data as Lua table

- **WHEN** a plugin calls `sift.jq.query({users={{name="Alice"}}}, '.users[].name')`
- **THEN** T3.11 SHALL convert the table to JSON, apply the filter, and return the result.
