# sift.* API

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Remove `ctx` parameter from `sift.str.split_lines()` Rust signature |
| T1.2 | Remove `ctx` parameter from `sift.str.slice_text()` Rust signature |
| T1.3 | Remove `ctx` parameter from `sift.str.is_sensitive()` Rust signature |
| T3.1 | Update README API reference for `sift.str.*` |

## MODIFIED Requirements

### Requirement: sift.str.* are pure functions

The system SHALL register `sift.str.split_lines`, `sift.str.slice_text`, and `sift.str.is_sensitive` as pure functions accepting only the data argument, without a `ctx` table parameter. ALWAYS T1.1, T1.2, T1.3 SHALL remove the `ctx` parameter from their Rust mlua function signatures.

#### Scenario: split_lines called without ctx

- **WHEN** a plugin calls `sift.str.split_lines("a\nb\nc")`
- **THEN** the system SHALL return `{"a", "b", "c"}`

#### Scenario: slice_text called without ctx

- **WHEN** a plugin calls `sift.str.slice_text("a\nb\nc\n", 2, 3)`
- **THEN** the system SHALL return `"b\nc"`

#### Scenario: is_sensitive called without ctx

- **WHEN** a plugin calls `sift.str.is_sensitive(".env")`
- **THEN** the system SHALL return `true`