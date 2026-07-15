# Store Primitive

## Purpose

Provide `sift.store(ctx, content, slug)` for explicit content storage with auto-nudge.

## Requirements

### Requirement: Store writes to session-scoped path

The system SHALL write content to `/tmp/sift/<session_id>/<timestamp>_<cmd_count>_<slug>`.

#### Scenario: Store creates file and nudges

- **WHEN** the system is called with content `"..."` and slug `"openspec-output.json"`
- **THEN** the system SHALL write the content to disk and emit `[sift] stored: 'command cat <path>'`.
