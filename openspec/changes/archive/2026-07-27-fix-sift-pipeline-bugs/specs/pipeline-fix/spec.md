## ADDED Requirements

### Requirement: Agent mode SHALL flush stdout before exit
`sift -c <command>` SHALL flush `std::io::stdout()` before calling `std::process::exit()` so that all output is visible to the caller.

#### Scenario: agent mode piped output is visible
- **WHEN** T1.1 runs `sift -c "echo '{\"a\":1}' | jq '.a'"`
- **THEN** the output SHALL contain `1` and the exit code SHALL be 0

### Requirement: Pipeline splitter SHALL respect shell quoting
`split_pipeline()` SHALL NOT split on `|` characters that appear inside single-quoted strings, double-quoted strings, or after a backslash escape.

#### Scenario: URL with pipe character is not split
- **WHEN** T2.1 processes `curl 'https://example.com?filter=a|b' | jq '.'`
- **THEN** the first segment SHALL be `curl 'https://example.com?filter=a|b'` (the pipe inside quotes is preserved)

#### Scenario: double-quoted pipe is not split
- **WHEN** T2.1 processes `echo "a|b" | cat`
- **THEN** the first segment SHALL be `echo "a|b"` (the pipe inside double quotes is preserved)

#### Scenario: escaped pipe is not split
- **WHEN** T2.1 processes `echo a\|b | cat`
- **THEN** the first segment SHALL be `echo a\|b` (the escaped pipe is preserved)

### Requirement: Unmatched pipeline segments SHALL fall back to bash
When `try_pipeline()` finds no plugin matching the last pipeline segment, it SHALL run the entire pipeline through `exec_command()` in bash instead of falling through to normal dispatch.

#### Scenario: curl piped to grep falls back to bash
- **WHEN** T3.1 processes `curl https://example.com 2>/dev/null | grep body`
- **THEN** the command SHALL execute correctly via bash and produce the expected grep output

#### Scenario: unmatched pipeline preserves exit code
- **WHEN** T3.1 processes `false | true`
- **THEN** the exit code SHALL be 0 (the exit code of the last command in the pipeline)

### Requirement: Existing pipeline optimization SHALL continue to work
When `try_pipeline()` finds a matching plugin for the last segment, it SHALL continue to use the existing optimization (run preceding in bash, pipe to plugin).

#### Scenario: jq plugin pipeline still works
- **WHEN** T4.1 processes `echo '{"a":1}' | jq '.a'`
- **THEN** the output SHALL contain `1` and the exit code SHALL be 0

#### Scenario: cat plugin pipeline still works
- **WHEN** T4.1 processes `echo hello | cat`
- **THEN** the output SHALL contain `hello`
