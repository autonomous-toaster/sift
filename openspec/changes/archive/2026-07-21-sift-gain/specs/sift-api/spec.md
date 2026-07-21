# sift.* API

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.3 | Add `sift.gain.report()` Rust function |
| T1.4 | Wire `register_gain()` into `register_sift_table()` |

## MODIFIED Requirements

### Requirement: sift.str.* are pure functions

The system SHALL register `sift.str.split_lines`, `sift.str.slice_text`, and `sift.str.is_sensitive` as pure functions accepting only the data argument, without a `ctx` table parameter.

#### Scenario: split_lines called without ctx

- **WHEN** a plugin calls `sift.str.split_lines("a\nb\nc")`
- **THEN** the system SHALL return `{"a", "b", "c"}`

#### Scenario: slice_text called without ctx

- **WHEN** a plugin calls `sift.str.slice_text("a\nb\nc\n", 2, 3)`
- **THEN** the system SHALL return `"b\nc"`

#### Scenario: is_sensitive called without ctx

- **WHEN** a plugin calls `sift.str.is_sensitive(".env")`
- **THEN** the system SHALL return `true`

## ADDED Requirements

### Requirement: sift.gain.report() queries session store

The system SHALL register `sift.gain.report` as a function taking a table of flags and returning a formatted report string. ALWAYS T1.3 SHALL complete BEFORE T1.4 SHALL wire the function into the `sift` table. ALWAYS T1.4 SHALL call `register_gain()` during `register_sift_table()`.

#### Scenario: Report called with no flags

- **WHEN** T1.4 calls `sift.gain.report({})`
- **THEN** the system SHALL return a formatted string with aggregate stats for the current session

#### Scenario: Report called with json flag

- **WHEN** T1.4 calls `sift.gain.report({json = true})`
- **THEN** the system SHALL return a JSON string with structured data