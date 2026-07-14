# sift.* API

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Implement `sift.exec(cmd)` — spawn bash via PTY, return output and exit code |
| T3.2 | Implement `sift.cache.{get,set,has}` — session-scoped key-value cache |
| T3.3 | Implement `sift.hash.{sha256,md5}` — hashing functions |
| T3.4 | Implement `sift.fs.{read,write,edit,stat,exists}` — filesystem operations |
| T3.5 | Implement `sift.json.{encode,decode}` — JSON encoding/decoding |
| T3.6 | Implement `sift.toon.{encode,decode}` — TOON encoding/decoding |
| T3.7 | Implement `sift.jq.query(data, filter)` — jq filter execution |
| T3.8 | Implement `sift.env.{get,set}` — environment variable access |
| T3.9 | Implement `sift.classify(cmd)` — command parsing and classification |
| T3.10 | Implement `sift.{log,exit,output}` — logging, exit, and output functions |
| T3.11 | Implement `sift.meta` — read-only context with writable raw_bytes |
| T3.12 | Implement `sift.token_count(text)` — token estimation |

## Requirements

### Requirement: sift.exec saves raw output

ALWAYS T3.1 SHALL save raw PTY output to `/tmp/sift/<session>/<cmd_count>_<slug>.log`.

#### Scenario: Raw output is saved

- **WHEN** T3.1 executes a command via PTY
- **THEN** the raw output SHALL be written to a temp file and the path SHALL be recorded in `sift.meta`.

### Requirement: sift.exec detects output format

ALWAYS T3.1 SHALL detect the output format from content.

#### Scenario: JSON format detected

- **WHEN** the raw output starts with `{` or `[`
- **THEN** T3.1 SHALL record the format as `json`.

### Requirement: sift.fs.read mirrors pi read tool

ALWAYS T3.4 SHALL accept `(path, {offset?, limit?})` matching pi's read tool signature.

#### Scenario: Read with offset and limit

- **WHEN** a plugin calls `sift.fs.read("file.rs", {offset=10, limit=50})`
- **THEN** T3.4 SHALL return lines 10-59 of the file.

### Requirement: sift.fs.edit mirrors pi edit tool

ALWAYS T3.4 SHALL accept `(path, edits)` where edits is an array of `{oldText, newText}` objects.

#### Scenario: Edit with multiple replacements

- **WHEN** a plugin calls `sift.fs.edit("file.rs", {{oldText="foo", newText="bar"}})`
- **THEN** T3.4 SHALL apply the replacement and return the diff.

### Requirement: sift.jq.query supports full jq syntax

ALWAYS T3.7 SHALL use the `jaq` crate to execute full jq filter syntax.

#### Scenario: Complex jq filter

- **WHEN** a plugin calls `sift.jq.query(data, 'map(select(.status == "FAILED")) | .[].name')`
- **THEN** T3.7 SHALL return the filtered JSON result as a string.

### Requirement: sift.meta.raw_bytes is writable

ALWAYS T3.11 SHALL allow plugins to set `sift.meta.raw_bytes` to override the default.

#### Scenario: Plugin reports larger raw output

- **WHEN** a plugin transforms JSON to TOON and sets `sift.meta.raw_bytes = 15000`
- **THEN** the token tracking SHALL use 15000 as the raw byte count.
