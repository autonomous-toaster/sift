## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Skip wildcard plugins in pipeline optimization |
| T1.2 | Forward stdin in execute_passthrough |
| T1.3 | Pass stdin from dispatch to execute_passthrough |

## ADDED Requirements

### Requirement: Wildcard plugins do not trigger pipeline optimization

`try_pipeline()` SHALL NOT trigger pipeline optimization when the matched plugin has `pattern = "*"`. ALWAYS T1.1 SHALL check the matched plugin's pattern before proceeding with optimization.

#### Scenario: Pipeline with wildcard-matched command

- **WHEN** a pipeline like `find . -name "*.rs" | sort` is dispatched
- **AND** `sort` matches only the wildcard plugin (`rtk.lua` with `pattern = "*"`)
- **THEN** `try_pipeline()` SHALL return `None` and the full pipeline SHALL run in bash

#### Scenario: Pipeline with specific plugin match

- **WHEN** a pipeline like `echo abc | cat` is dispatched
- **AND** `cat` matches a specific plugin (`cat.lua` with `pattern = "cat"`)
- **THEN** `try_pipeline()` SHALL proceed with optimization as before

### Requirement: Passthrough preserves pipeline stdin

When `dispatch()` receives stdin from pipeline optimization and the matched plugin returns `"passthrough"`, the stdin SHALL be forwarded to `execute_passthrough()`. ALWAYS T1.2 and T1.3 SHALL ensure the accumulated stdin from preceding pipeline segments is passed to the passthrough command.

#### Scenario: Passthrough with piped input

- **WHEN** a specific plugin returns `{ status = "passthrough" }` in pipeline mode
- **AND** stdin contains data from preceding pipeline segments
- **THEN** `execute_passthrough()` SHALL receive that stdin data
- **AND** the passthrough command SHALL process it as if the full pipeline ran in bash

#### Scenario: Passthrough without piped input

- **WHEN** `dispatch()` is called without stdin (not in pipeline mode)
- **AND** the plugin returns `{ status = "passthrough" }`
- **THEN** `execute_passthrough()` SHALL behave as before with empty stdin
