# Openspec Plugin — --json Injection and TOON Conversion

## Task Reference

| Task ID | Description |
|---------|-------------|
| T6.1 | Write `plugins/openspec.lua` — inject --json, convert to toon via sift.json.shortest() |

## ADDED Requirements

### Requirement: Openspec plugin injects --json flag

ALWAYS T6.1 SHALL check if `--json` is present in the command arguments and append it if missing.

#### Scenario: --json injected automatically

- **WHEN** a user runs `openspec list`
- **THEN** T6.1 SHALL execute `openspec list --json` instead.

#### Scenario: Existing --json preserved

- **WHEN** a user runs `openspec list --json`
- **THEN** T6.1 SHALL NOT add a second --json flag.

### Requirement: Output converted via sift.json.shortest()

ALWAYS T6.1 SHALL pass JSON output through `sift.json.shortest(ctx, output, {json={max_string_len=80}, toon={}})`.

#### Scenario: TOON selected for large JSON

- **WHEN** T6.1 receives 10,000 tokens of JSON output and TOON is the shortest representation
- **THEN** T6.1 SHALL return TOON output with auto-nudge for raw original.
