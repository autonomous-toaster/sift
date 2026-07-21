## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.4 | Add mdmin dep, implement sift.ext.markdown module |
| T1.5 | Register sift.ext table and all sub-modules in Lua API |

## ADDED Requirements

### Requirement: Markdown compression

T1.4 SHALL complete BEFORE T1.5 SHALL run.

#### Scenario: Markdown compressed at level 2

- **WHEN** T1.4 SHALL complete
- **THEN** `sift.ext.markdown.compress("# Title\n\nSome **bold** text", {level=2})` SHALL return a string shorter than the input

#### Scenario: Level 0 returns unchanged

- **WHEN** T1.4 SHALL complete
- **THEN** `sift.ext.markdown.compress("hello", {level=0})` SHALL return `"hello"`

### Requirement: Feature flag gating

T1.5 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: markdown available when feature enabled

- **WHEN** T1.5 SHALL complete AND the `mdmin` feature SHALL be enabled
- **THEN** `sift.ext.markdown` SHALL NOT be nil

#### Scenario: markdown nil when feature disabled

- **WHEN** T1.5 SHALL complete AND the `mdmin` feature SHALL NOT be enabled
- **THEN** `sift.ext.markdown` SHALL be nil
