## ADDED Requirements

### Requirement: Commands with `$` SHALL expand variables correctly
When a command containing `$` (variable expansion or command substitution) is dispatched through sift, the shell variables SHALL be expanded by bash, not treated as literal strings.

#### Scenario: simple variable expansion
- **WHEN** T1.1 dispatches `echo "$HOME"` through `dispatch_full()`
- **THEN** the output SHALL contain the expanded value of `$HOME` (e.g., `/Users/...`), not the literal string `$HOME`

#### Scenario: variable in passthrough plugin
- **WHEN** T1.2 a plugin returns `status = "passthrough"` for a command containing `$VAR`
- **THEN** the passthrough execution SHALL use the original command string, not a reconstructed one

### Requirement: In-process plugins SHALL continue to work
Plugins that handle commands in-process (without passing to bash) SHALL NOT be affected by this change.

#### Scenario: jq plugin with variable in pipeline
- **WHEN** T2.1 dispatches `echo '{"a":1}' | jq '.a'` (no `$` involved)
- **THEN** the jq plugin SHALL handle it as before, output `[1]`

#### Scenario: cat plugin with file path
- **WHEN** T2.1 dispatches `cat /tmp/test.txt`
- **THEN** the cat plugin SHALL handle it as before

### Requirement: Pipeline optimization SHALL continue to work
The pipeline optimization in `try_pipeline()` SHALL NOT be affected.

#### Scenario: piped command with plugin match
- **WHEN** T3.1 dispatches `echo '{"a":1}' | jq '.a'`
- **THEN** the pipeline optimization SHALL handle it as before, output `[1]`
