# jq-plugin

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Create jq.lua plugin file |
| T1.2 | Add extension fallback to curl.lua |
| T1.3 | Fix README jaq CLI reference |

## ADDED Requirements

### Requirement: jq plugin intercepts jq commands

The jq plugin SHALL intercept commands matching the `jq` pattern and process them through the in-process `jaq` library via `sift.jq.query()`.

#### Scenario: Basic filter with piped stdin

- **WHEN** the system processes `curl ... | jq '.results[] \| {title, url}'` and the jq plugin matches
- **THEN** the system SHALL read piped stdin, apply the filter via `sift.jq.query()`, and compress the output with `sift.json.shortest()`.

#### Scenario: No piped stdin and no -n flag

- **WHEN** the jq plugin receives a command with no piped stdin and no `-n` flag
- **THEN** the system SHALL fall through to real `jq`.

### Requirement: jq plugin handles -r flag

The jq plugin SHALL support the `-r` (raw output) flag by decoding the JSON result and extracting raw string values.

#### Scenario: Raw output with strings

- **WHEN** the system processes `jq -r '.name'` with piped JSON input `[{"name":"John"},{"name":"Jane"}]`
- **THEN** the system SHALL output `John\nJane` as raw text, and `sift.json.shortest()` SHALL return it unchanged.

#### Scenario: Raw output with objects

- **WHEN** the system processes `jq -r '. \| {name, age}'` with piped JSON input
- **THEN** the system SHALL decode the JSON array, extract each object as a string, and join with newlines.

### Requirement: jq plugin handles -n flag

The jq plugin SHALL support the `-n` (null input) flag by passing `"null"` as the JSON input to `sift.jq.query()`.

#### Scenario: Null input constructs JSON

- **WHEN** the system processes `jq -n '{a: 1, b: 2}'`
- **THEN** the system SHALL pass `"null"` as input to `sift.jq.query()` and return the compressed result.

### Requirement: jq plugin falls through on unknown flags

The jq plugin SHALL fall through to real `jq` when it encounters flags it cannot handle.

#### Scenario: Unknown flag

- **WHEN** the system processes `jq --arg name foo '.name'` with an `--arg` flag
- **THEN** the system SHALL return `{ status = "passthrough" }` to run the real `jq` command.

#### Scenario: -f flag (from-file)

- **WHEN** the system processes `jq -f filter.jq data.json`
- **THEN** the system SHALL return `{ status = "passthrough" }` to run the real `jq` command.

### Requirement: jq plugin compresses output with shortest format

The jq plugin SHALL always pass the result through `sift.json.shortest()` with `{toon = true}` to select the most token-efficient format.

#### Scenario: JSON output compressed

- **WHEN** the jq plugin produces a JSON result from `sift.jq.query()`
- **THEN** the system SHALL call `sift.json.shortest(ctx, result, {toon = true})` and return the compressed output.

#### Scenario: Raw output passes through

- **WHEN** the jq plugin produces raw text output (via `-r`)
- **THEN** `sift.json.shortest()` SHALL return the raw text unchanged.

### Requirement: curl plugin checks URL extension

The curl plugin SHALL check the URL extension as a fallback when the content-type is not recognized as JSON, HTML, or a supported document format.

#### Scenario: .md file served as text/plain

- **WHEN** the curl plugin receives a response with content-type `text/plain` and the URL ends in `.md`
- **THEN** the system SHALL apply mdmin compression via `sift.ext.markdown.compress()` if available.

#### Scenario: .json file served as text/plain

- **WHEN** the curl plugin receives a response with content-type `text/plain` and the URL ends in `.json`
- **THEN** the system SHALL apply `sift.json.shortest()` compression.

#### Scenario: Content-type takes priority

- **WHEN** the curl plugin receives a response with content-type `application/json` and the URL ends in `.md`
- **THEN** the system SHALL use the content-type detection (JSON → TOON), not the extension.
