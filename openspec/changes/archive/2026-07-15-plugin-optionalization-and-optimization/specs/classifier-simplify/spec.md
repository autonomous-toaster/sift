# Classifier Simplification — Remove Dead CommandKind, Keep General Parsing

## Task Reference

| Task ID | Description |
|---------|-------------|
| T7.1 | Remove `CommandKind` enum and `classify_command()` function from classifier.rs |
| T7.2 | Simplify `Classification` struct to `{name, args, is_piped, is_compound}` without `kind` |
| T7.3 | Update `sift.classify()` Lua binding to return simplified struct |
| T7.4 | Remove `classifier.rs` tests referencing specific CommandKind variants |

## MODIFIED Requirements

### Requirement: Classify returns name, args, is_piped, is_compound

ALWAYS T7.2 SHALL return a table with `name`, `args`, `is_piped`, and `is_compound` fields. The `kind` field SHALL be removed.

#### Scenario: Simple command classified

- **WHEN** T7.2 classifies `"docker ps"`
- **THEN** T7.2 SHALL return `{name="docker", args={"ps"}, is_piped=false, is_compound=false}`.

#### Scenario: Compound command detected

- **WHEN** T7.2 classifies `"cd /x && docker ps"`
- **THEN** T7.2 SHALL return `{name="cd", args={"/x", "&&", "docker", "ps"}, is_piped=false, is_compound=true}`.
