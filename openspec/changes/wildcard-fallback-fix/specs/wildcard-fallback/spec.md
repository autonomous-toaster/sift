## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Split `find_plugin()` into two passes: specific patterns first, wildcard fallback second |

## ADDED Requirements

### Requirement: Specific patterns checked before wildcard

T1.1 SHALL complete BEFORE wildcard fallback SHALL run.

#### Scenario: Cat beats wildcard for cat Cargo.toml
- **WHEN** T1.1 dispatches `"cat Cargo.toml"` with candidates `["cat", "cat Cargo.toml"]`
- **THEN** T1.1 SHALL match `"cat"` against cat.lua's pattern before considering `"*"`.

### Requirement: Wildcard catches unmatched commands

ALWAYS T1.1 SHALL fall back to the plugin with pattern `"*"` when no specific pattern matches any candidate.

#### Scenario: Wildcard catches unmatched commands
- **WHEN** T1.1 dispatches `"docker ps"` and no plugin has pattern `"docker"` or `"docker ps"`
- **THEN** T1.1 SHALL fall back to the plugin with pattern `"*"`.
