## Purpose

Ensure specific plugin patterns are matched before wildcard (`*`) patterns, so dedicated plugins like `cat` take precedence over the catch-all wildcard plugin.

## Requirements

### Requirement: Specific patterns checked before wildcard

The `find_plugin()` function SHALL use two passes: specific patterns first, wildcard fallback second. When dispatching a command, specific patterns SHALL be checked against plugin candidates before considering the `*` wildcard pattern.

#### Scenario: Cat beats wildcard for cat Cargo.toml

- **WHEN** dispatching `"cat Cargo.toml"` with candidates `["cat", "cat Cargo.toml"]`
- **THEN** the system SHALL match `"cat"` against cat.lua's pattern before considering `"*"`

### Requirement: Wildcard catches unmatched commands

The system SHALL fall back to the plugin with pattern `"*"` when no specific pattern matches any candidate.

#### Scenario: Wildcard catches unmatched commands

- **WHEN** dispatching `"docker ps"` and no plugin has pattern `"docker"` or `"docker ps"`
- **THEN** the system SHALL fall back to the plugin with pattern `"*"`