# Sift Read Diff

## Purpose

Wire up diff emission in the sift-read plugin: on cache miss, look up old content by path hash, compute diff, and emit if useful.

## Requirements

### Requirement: sift-read emits diff on cache miss

The system SHALL compute a unified diff when a file's content hash differs from the cached hash, and emit it if the diff is significantly smaller than the full content.

#### Scenario: Diff emitted when file changes
- **WHEN** the system reads a file whose content hash differs from the cached hash
- **THEN** the system SHALL compute a unified diff and return it if the diff is smaller than 90% of the full content.

#### Scenario: Full content returned when diff is too large
- **WHEN** the system reads a file whose content hash differs and the diff is larger than 90% of the new content
- **THEN** the system SHALL return the full content instead of the diff.
