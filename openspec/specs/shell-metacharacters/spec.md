# Shell Metacharacter Handling

## Purpose

Handle shell metacharacters (`;`, `&&`, `||`) in command dispatch so that commands like `wc -c; echo $?` are split correctly: the first segment goes to plugin matching, the rest goes to bash passthrough. This prevents plugins from reconstructing commands with metacharacters inside single quotes, which breaks shell variable expansion.

## Requirements

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
