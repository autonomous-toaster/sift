# Openspec Plugin

## Purpose

Provide a built-in openspec.lua plugin that injects `--json` flag and converts output via `sift.json.shortest()`.

## Requirements

### Requirement: Openspec plugin injects --json flag

The system SHALL check if `--json` is present in the command arguments and append it if missing.

#### Scenario: --json injected automatically

- **WHEN** a user runs `openspec list`
- **THEN** the system SHALL execute `openspec list --json` instead.

#### Scenario: Existing --json preserved

- **WHEN** a user runs `openspec list --json`
- **THEN** the system SHALL NOT add a second --json flag.

### Requirement: Output converted via sift.json.shortest()

The system SHALL pass JSON output through `sift.json.shortest(ctx, output, {json={max_string_len=80}, toon={}})`.

#### Scenario: TOON selected for large JSON

- **WHEN** the system receives 10,000 tokens of JSON output and TOON is the shortest representation
- **THEN** the system SHALL return TOON output with auto-nudge for raw original.
