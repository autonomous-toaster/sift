# sift.* API

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Rewrite `sift.toon.encode` to accept optional options table with `delimiter` and `indent` |
| T1.2 | Rewrite `sift.toon.decode` to accept optional options table with `strict` and `no_coerce` |
| T1.3 | Drop `ctx` parameter from both functions |
| T2.1 | Update smoke test to verify new signatures |
| T2.2 | Verify no shipped plugins break |

## MODIFIED Requirements

### Requirement: sift.toon.encode accepts options

The system SHALL register `sift.toon.encode` as a function taking `data` and an optional `options` table. ALWAYS T1.1 SHALL complete BEFORE T1.3 SHALL drop the `ctx` parameter. ALWAYS T1.1 SHALL map `options.delimiter` ("comma"|"pipe") to `Delimiter` and `options.indent` ("tab"|"space2"|"space4") to `Indent`. When no options are passed, the system SHALL use `encode_default` (current behavior).

#### Scenario: encode without options

- **WHEN** T1.1 calls `sift.toon.encode({name = "Alice"})`
- **THEN** the system SHALL return the same output as `encode_default`

#### Scenario: encode with pipe delimiter

- **WHEN** T1.1 calls `sift.toon.encode({tags = {"a", "b"}}, {delimiter = "pipe"})`
- **THEN** the system SHALL use pipe-delimited TOON format

#### Scenario: encode with tab indent

- **WHEN** T1.1 calls `sift.toon.encode({name = "Alice"}, {indent = "tab"})`
- **THEN** the system SHALL use tab-indented TOON format

### Requirement: sift.toon.decode accepts options

The system SHALL register `sift.toon.decode` as a function taking a `str` and an optional `options` table. ALWAYS T1.2 SHALL complete BEFORE T1.3 SHALL drop the `ctx` parameter. ALWAYS T1.2 SHALL support `options.strict` (calls `decode_strict`) and `options.no_coerce` (calls `decode_no_coerce`). When both `strict` and `no_coerce` are true, the system SHALL return an error. When no options are passed, the system SHALL use `decode_default`.

#### Scenario: decode without options

- **WHEN** T1.2 calls `sift.toon.decode("name: Alice")`
- **THEN** the system SHALL return the same output as `decode_default`

#### Scenario: decode with strict mode

- **WHEN** T1.2 calls `sift.toon.decode("items[3]: a,b", {strict = true})`
- **THEN** the system SHALL validate array bounds strictly

#### Scenario: decode with no_coerce

- **WHEN** T1.2 calls `sift.toon.decode("count: 42", {no_coerce = true})`
- **THEN** the system SHALL return `"42"` as a string, not a number

#### Scenario: strict and no_coerce conflict

- **WHEN** T1.2 calls `sift.toon.decode("x: 1", {strict = true, no_coerce = true})`
- **THEN** the system SHALL return an error string

### Requirement: ctx parameter removed

The system SHALL register `sift.toon.encode` and `sift.toon.decode` without a `ctx` parameter. ALWAYS T1.3 SHALL complete AFTER T1.1 and T1.2. The functions SHALL be pure, like `sift.str.*`.

#### Scenario: encode called without ctx

- **WHEN** a plugin calls `sift.toon.encode({key = "val"})`
- **THEN** the system SHALL return the TOON-encoded string

#### Scenario: decode called without ctx

- **WHEN** a plugin calls `sift.toon.decode("key: val")`
- **THEN** the system SHALL return the decoded Lua table