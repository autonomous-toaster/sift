# Shell Metacharacter Handling

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `split_first_command()` function |
| T1.2 | Modify `dispatch_full()` to use `split_first_command()` |
| T2.1 | Unit tests for `split_first_command()` |
| T2.2 | Integration test for shell metacharacter handling |

## ADDED Requirements

### Requirement: split-first-command

T1.1 SHALL complete BEFORE T1.2 SHALL start.

#### Scenario: split-first-command-completes

**WHEN** T1.1 runs
**THEN** `split_first_command()` SHALL return the first command segment and the rest

### Requirement: dispatch-uses-split

T1.2 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: dispatch-splits-metachar-commands

**WHEN** T1.2 runs
**THEN** `dispatch_full()` SHALL split commands on `;`, `&&`, `||` outside quotes

### Requirement: tests-validate

T2.1 SHALL complete AFTER T1.1 SHALL complete.
T2.2 SHALL complete AFTER T1.2 SHALL complete.

#### Scenario: unit-tests-pass

**WHEN** T2.1 runs
**THEN** unit tests SHALL pass for all metacharacter combinations

#### Scenario: integration-tests-pass

**WHEN** T2.2 runs
**THEN** integration tests SHALL pass for shell metacharacter handling
