# Veriplan Plugin

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Fix nudge message length calculation in `json_shortest_impl()` |
| T2.1 | Create `plugins/veriplan.lua` |
| T3.1 | Unit test for nudge overhead calculation |
| T3.2 | Integration test for veriplan plugin |

## ADDED Requirements

### Requirement: nudge-overhead-fix

T1.1 SHALL complete BEFORE T2.1 SHALL start.

#### Scenario: nudge-overhead-correct

**WHEN** T1.1 runs
**THEN** `json_shortest_impl()` SHALL use the actual nudge message prefix length instead of a hardcoded value

### Requirement: veriplan-plugin-created

T2.1 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: veriplan-plugin-intercepts-check

**WHEN** T2.1 runs
**THEN** the veriplan plugin SHALL intercept `veriplan check` commands and ensure `--json` output

### Requirement: tests-validate

T3.1 SHALL complete AFTER T1.1 SHALL complete.
T3.2 SHALL complete AFTER T2.1 SHALL complete.

#### Scenario: unit-tests-pass

**WHEN** T3.1 runs
**THEN** unit tests SHALL verify the nudge overhead calculation

#### Scenario: integration-tests-pass

**WHEN** T3.2 runs
**THEN** integration tests SHALL verify veriplan plugin output compression
