# Streaming Transform

## Purpose

Allow plugins to transform output chunks in real-time via an optional `transform` callback on `sift.exec()`.

## Requirements

### Requirement: sift.exec accepts transform callback

The system SHALL accept an optional `{ transform = fn }` parameter on `sift.exec()` that transforms each stdout chunk before writing and collecting.

#### Scenario: Transform applied to each chunk
- **WHEN** the system runs `echo hello` with `{ transform = function(c) return string.upper(c) end }`
- **THEN** the system SHALL write `"HELLO\n"` to stdout and return `"HELLO\n"`.

### Requirement: Transform is optional

The system SHALL stream raw output when no transform is provided.

#### Scenario: No transform, raw streaming
- **WHEN** the system runs `echo hello` without a transform option
- **THEN** the system SHALL write `"hello\n"` to stdout and return `("hello\n", "", 0)`.
