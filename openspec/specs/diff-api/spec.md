# Diff API

## Purpose

Provide unified diff computation between two text strings via the `similar` crate, exposed to Lua plugins as `sift.diff()`.

## Requirements

### Requirement: sift.diff returns unified diff

The system SHALL compute a unified diff between two text strings with 3 lines of context.

#### Scenario: Diff between two texts
- **WHEN** the system computes diff between `"line1\nline2\n"` and `"line1\nline2 modified\n"`
- **THEN** the system SHALL return a unified diff showing the changed line.
