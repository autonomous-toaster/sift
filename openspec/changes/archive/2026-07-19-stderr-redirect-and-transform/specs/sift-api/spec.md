# sift.* API

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `parse_fd_redirects()` — detect `N>&M` patterns, return merge map |
| T1.2 | Wire fd redirect detection into `dispatch_with_redirect` — strip from args, set `merge_stderr` flag |
| T1.3 | Add `merge_stderr` param to `exec_command` |
| T1.4 | Add `merge_stderr` option to `sift.exec` |
| T1.5 | Pipeline deferral — skip redirect handling when `|` detected |
| T2.1 | Fix `bash.lua` to return `output` in result table |
| T3.1 | Update tests |

## MODIFIED Requirements

### Requirement: sift.exec supports merge_stderr

The system SHALL support a `merge_stderr` option in `sift.exec`. ALWAYS T1.4 SHALL pass the option from the Lua call to `exec_command`. When `merge_stderr` is true, the system SHALL apply transforms to stdout only, then append stderr raw to the output.

#### Scenario: merge_stderr with transform

- **WHEN** a plugin calls `sift.exec(ctx, cmd, {transform = fn, merge_stderr = true})`
- **THEN** the system SHALL apply `fn` to stdout chunks only, then append stderr to the transformed output

### Requirement: dispatch_with_redirect handles fd redirects

The system SHALL detect `N>&M` fd redirects in command arguments. ALWAYS T1.1 SHALL parse args matching `/^\d+>&\d+$/`. ALWAYS T1.2 SHALL strip matched args and set `merge_stderr = true` for `2>&1`. ALWAYS T1.5 SHALL skip all redirect handling when a pipe (`|`) is detected in the command.

#### Scenario: 2>&1 detected and stripped

- **WHEN** T1.2 parses args containing `"2>&1"`
- **THEN** the system SHALL remove `"2>&1"` from args and set `merge_stderr = true`

#### Scenario: 1>&2 detected and stripped

- **WHEN** T1.2 parses args containing `"1>&2"`
- **THEN** the system SHALL remove `"1>&2"` from args

#### Scenario: pipeline defers to bash

- **WHEN** T1.5 detects a pipe (`|`) in the command
- **THEN** the system SHALL skip all redirect handling and let bash handle it

### Requirement: bash.lua returns output with streamed flag

The system SHALL return `output` in the bash.lua result table with `streamed = true`. ALWAYS T2.1 SHALL set `output` to the command's stdout (transformed if applicable) plus stderr (if merge_stderr). When `streamed = true`, `dispatch` SHALL skip printing the output (it was already streamed by `exec_command`).

#### Scenario: bash.lua returns output with streamed flag

- **WHEN** T2.1 runs a command via bash.lua
- **THEN** the system SHALL return `output` and `streamed = true` in the result table