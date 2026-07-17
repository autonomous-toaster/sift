## Purpose

Replace `shQuote(command)` with `JSON.stringify(command)` in the spawnHook function to fix double-quoting bugs with paths containing single quotes.

## Requirements

### Requirement: spawnHook in sift.ts

The spawnHook function SHALL use `JSON.stringify(command)` instead of `shQuote(command)`. `JSON.stringify` is safe: no `/` escaping in Node.js, `$`/`` ` `` expansion inside double quotes is desired. `<`/`>` are literal inside double quotes (redirects handled by `dispatch_full`). Single quotes in command are literal (no double-quoting bug). The `siftExec` function already uses `JSON.stringify` — no change needed.

#### Scenario: Simple command works

- **WHEN** `bash("cat Justfile")` is called
- **THEN** it SHALL work correctly

#### Scenario: Variable expansion works

- **WHEN** `bash("echo $HOME")` is called
- **THEN** it SHALL expand `$HOME` correctly

#### Scenario: Redirect with sed works

- **WHEN** `bash("sed -n '1,10p' < Justfile")` is called
- **THEN** it SHALL work (redirect handled by dispatch_full)

#### Scenario: No double-quoting bug

- **WHEN** a path containing single quotes is used
- **THEN** there SHALL be no double-quoting bug