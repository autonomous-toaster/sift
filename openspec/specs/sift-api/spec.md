# sift.* API

## Purpose

Define the modified sift.* API signatures including ctx-first, nudge, store, and auto-nudge on exec error.

## Requirements

### Requirement: sift.log.nudge accumulates messages

The system SHALL accept a message string and accumulate it in the nudge buffer for the current plugin execution.

#### Scenario: Nudge added during plugin execution

- **WHEN** a plugin calls `sift.log.nudge(ctx, "json compressed")`
- **THEN** the system SHALL append `"json compressed"` to the nudge accumulator.

### Requirement: sift.store writes to disk

The system SHALL write content to `/tmp/sift/<session>/<ts>_<count>_<slug>` and emit a nudge with the path.

#### Scenario: Store with custom slug

- **WHEN** a plugin calls `sift.store(ctx, content, "my-output.json")`
- **THEN** the system SHALL write content to a file named with the slug and emit `[sift] stored: 'command cat <path>'`.

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

### Requirement: sift.exec saves on error only

The system SHALL save raw combined output to `/tmp/sift/<session>/<ts>_<count>_<slug>.log` only when `exit_code != 0`. On success (exit_code == 0), raw output SHALL NOT be saved.

#### Scenario: Error output saved and nudged

- **WHEN** the system executes a command that exits with code 1
- **THEN** the system SHALL save raw output to disk and emit `[sift] raw: 'command cat <path>'`.

#### Scenario: Success output not saved

- **WHEN** the system executes a command that exits with code 0
- **THEN** the system SHALL NOT save raw output to disk and SHALL NOT emit an error nudge.

### Requirement: sift.toon.encode accepts options

The system SHALL register `sift.toon.encode` as a function taking `data` and an optional `options` table. ALWAYS T1.1 SHALL map `options.delimiter` ("comma"|"pipe") to `Delimiter` and `options.indent` ("tab"|"space2"|"space4") to `Indent`. When no options are passed, the system SHALL use `encode_default`.

#### Scenario: encode without options

- **WHEN** a plugin calls `sift.toon.encode({name = "Alice"})`
- **THEN** the system SHALL return the same output as `encode_default`

#### Scenario: encode with pipe delimiter

- **WHEN** a plugin calls `sift.toon.encode({tags = {"a", "b"}}, {delimiter = "pipe"})`
- **THEN** the system SHALL use pipe-delimited TOON format

### Requirement: sift.toon.decode accepts options

The system SHALL register `sift.toon.decode` as a function taking a `str` and an optional `options` table. ALWAYS T1.2 SHALL support `options.strict` (calls `decode_strict`) and `options.no_coerce` (calls `decode_no_coerce`). When both `strict` and `no_coerce` are true, the system SHALL return an error. When no options are passed, the system SHALL use `decode_default`.

#### Scenario: decode without options

- **WHEN** a plugin calls `sift.toon.decode("name: Alice")`
- **THEN** the system SHALL return the decoded Lua table

#### Scenario: decode with strict mode

- **WHEN** a plugin calls `sift.toon.decode("items[3]: a,b", {strict = true})`
- **THEN** the system SHALL validate array bounds strictly

#### Scenario: strict and no_coerce conflict

- **WHEN** a plugin calls `sift.toon.decode("x: 1", {strict = true, no_coerce = true})`
- **THEN** the system SHALL return an error string

### Requirement: sift.toon.* are pure functions

The system SHALL register `sift.toon.encode` and `sift.toon.decode` without a `ctx` parameter. The functions SHALL be pure, like `sift.str.*`.

#### Scenario: encode called without ctx

- **WHEN** a plugin calls `sift.toon.encode({key = "val"})`
- **THEN** the system SHALL return the TOON-encoded string

#### Scenario: decode called without ctx

- **WHEN** a plugin calls `sift.toon.decode("key: val")`
- **THEN** the system SHALL return the decoded Lua table

### Requirement: sift.gain.report() queries session store

The system SHALL register `sift.gain.report` as a function taking a table of flags and returning a formatted report string. The `--gain` CLI flag SHALL be the primary entry point.

#### Scenario: CLI --gain prints report

- **WHEN** a user runs `sift --gain`
- **THEN** the system SHALL print the gain report to stdout and exit

#### Scenario: record_conversation without tokio runtime

- **WHEN** `record_conversation` is called outside a tokio context
- **THEN** the system SHALL spawn a new thread with a dedicated runtime instead of panicking
