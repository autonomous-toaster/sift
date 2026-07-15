# Path Hash Tracking

## Purpose

Track the last known content hash for each file path, enabling lookup of old content on cache miss for diff computation.

## Requirements

### Requirement: Path-to-hash mapping persists

The system SHALL persist `path → last_content_hash` mappings across invocations within the same `AI_SESSION`.

#### Scenario: Set and get path hash
- **WHEN** the system sets path hash for `"/tmp/test.txt"` to `"abc123"`
- **THEN** the system SHALL return `"abc123"` when getting path hash for `"/tmp/test.txt"`.

#### Scenario: Unknown path returns nil
- **WHEN** the system gets path hash for an unknown path
- **THEN** the system SHALL return nil.
