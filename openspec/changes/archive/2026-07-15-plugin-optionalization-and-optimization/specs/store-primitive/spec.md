# Store Primitive — Explicit Content Storage with Auto-Nudge

## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Implement `sift.store(ctx, content, slug)` — write content to /tmp/sift/<session>/, return path, emit nudge |

## ADDED Requirements

### Requirement: Store writes to session-scoped path

ALWAYS T3.1 SHALL write content to `/tmp/sift/<session_id>/<timestamp>_<cmd_count>_<slug>`.

#### Scenario: Store creates file and nudges

- **WHEN** T3.1 is called with content `"..."` and slug `"openspec-output.json"`
- **THEN** T3.1 SHALL write the content to disk and emit `[sift] use 'command cat <path>' for openspec-output.json`.
