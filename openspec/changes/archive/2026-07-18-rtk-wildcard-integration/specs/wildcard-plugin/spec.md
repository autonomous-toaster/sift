## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `"*"` wildcard matching in `find_plugin()` |
| T2.1 | Change rtk.lua pattern from hardcoded list to `"*"` |

## ADDED Requirements

### Requirement: Wildcard support before rtk uses it

T1.1 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Wildcard available for rtk
- **WHEN** T2.1 runs
- **THEN** T1.1 SHALL have completed, making `"*"` pattern available in `find_plugin()`.

### Requirement: Specific pattern beats wildcard

ALWAYS T1.1 SHALL prefer a plugin with a longer matching pattern over a plugin with `"*"` when both match the same candidate.

#### Scenario: Cat beats wildcard
- **WHEN** T1.1 dispatches `"cat foo.rs"` and a plugin has `pattern = "cat"` and another has `pattern = "*"`
- **THEN** T1.1 SHALL select the plugin with `pattern = "cat"`.
