## ADDED Requirements

### Requirement: Pipeline error output is displayed

When sift executes a piped command via `sift -c` and the preceding command exits with a non-zero code, the combined stdout and stderr from the preceding command SHALL be displayed to the user.

T1.1 SHALL complete BEFORE T1.2 SHALL run.

#### Scenario: Preceding command fails with stderr
- **WHEN** user runs `sift -c 'grep foo nonexistent.txt | head -5'`
- **THEN** sift SHALL display the grep error message and exit with grep's exit code

#### Scenario: Preceding command fails with stdout
- **WHEN** user runs `sift -c 'grep foo file.txt | head -5'` and grep finds matches but exits non-zero (e.g., due to a warning)
- **THEN** sift SHALL display grep's stdout matches

### Requirement: stderr forwarded on pipeline success

When the pipeline optimization runs and the preceding command exits with code 0, its stderr SHALL be included in the input passed to the last segment's plugin.

T1.2 SHALL complete BEFORE T1.3 SHALL run.

#### Scenario: stderr from preceding command visible
- **WHEN** user runs `sift -c 'grep foo file.txt 2>&1 | head -5'` and grep writes a warning to stderr
- **THEN** the warning SHALL appear in the output

### Requirement: EPIPE errors handled gracefully

When `exec_command` writes stdin data to a child process and the child closes its stdin early (EPIPE), the error SHALL be handled gracefully without data loss.

T1.3 SHALL complete BEFORE T1.4 SHALL run.

#### Scenario: head closes stdin early
- **WHEN** sift runs `head -5` with a large stdin payload
- **THEN** sift SHALL NOT crash or produce truncated output due to EPIPE
