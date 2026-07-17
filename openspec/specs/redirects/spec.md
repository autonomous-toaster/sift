## Purpose

Handle shell redirects (`< file`, `> file`, `>> file`) inside `dispatch_full()` so that common patterns work without falling through to the shell.

## Requirements

### Requirement: Redirect parsing in dispatch_full()

The system SHALL parse `< file` from args: read file content, pass as stdin to plugin, strip `<` and file from args. It SHALL parse `> file` from args: capture plugin output, write to file, strip `>` and file from args. It SHALL parse `>> file` from args: capture plugin output, append to file, strip `>>` and file from args. Complex redirects (`2>`, `&>`, heredocs, `<<<`) SHALL fall through to the shell. Redirect parsing SHALL happen after pipeline handling, before normal dispatch.

#### Scenario: Input redirect

- **WHEN** `sed -n '1,10p' < Justfile` is called
- **THEN** it SHALL return lines 1-10 (no crash)

#### Scenario: Output redirect

- **WHEN** `echo hello > /tmp/out` is called
- **THEN** it SHALL write "hello" to /tmp/out

#### Scenario: Append redirect

- **WHEN** `echo hello >> /tmp/out` is called
- **THEN** it SHALL append "hello" to /tmp/out

#### Scenario: Complex redirect falls through

- **WHEN** `echo hello 2> /tmp/err` is called
- **THEN** it SHALL fall through to the shell