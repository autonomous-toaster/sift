# Classifier Simplification

## Purpose

Simplify command classification by removing the `CommandKind` enum (dead code) while keeping general parsing infrastructure.

## Requirements

### Requirement: Classify returns name, args, is_piped, is_compound

The system SHALL return a table with `name`, `args`, `is_piped`, and `is_compound` fields. The `kind` field SHALL be removed.

#### Scenario: Simple command classified

- **WHEN** the system classifies `"docker ps"`
- **THEN** the system SHALL return `{name="docker", args={"ps"}, is_piped=false, is_compound=false}`.

#### Scenario: Compound command detected

- **WHEN** the system classifies `"cd /x && docker ps"`
- **THEN** the system SHALL return `{name="cd", args={"/x", "&&", "docker", "ps"}, is_piped=false, is_compound=true}`.
