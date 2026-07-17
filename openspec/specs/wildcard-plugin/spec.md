## Purpose

Support wildcard (`"*"`) pattern matching in `find_plugin()` so catch-all plugins like rtk can match any command while specific plugins still take precedence.

## Requirements

### Requirement: Wildcard support in find_plugin()

The `find_plugin()` function SHALL support `"*"` as a wildcard pattern that matches any command candidate. When a plugin has pattern `"*"`, it SHALL match any candidate.

### Requirement: Specific pattern beats wildcard

The system SHALL prefer a plugin with a longer matching pattern over a plugin with `"*"` when both match the same candidate.

#### Scenario: Cat beats wildcard

- **WHEN** dispatching `"cat foo.rs"` and a plugin has `pattern = "cat"` and another has `pattern = "*"`
- **THEN** the system SHALL select the plugin with `pattern = "cat"`