## ADDED Requirements

### Requirement: cat plugin handles heredoc syntax safely
The cat plugin SHALL passthrough to bash when the path argument contains shell metacharacters (`<`, `>`, `|`, `;`, `&`, `` ` ``) or does not exist as a file.

#### Scenario: Heredoc syntax passthrough
- **WHEN** the agent runs `cat << 'EOF'`
- **THEN** the cat plugin SHALL passthrough to bash instead of attempting to stat the path

#### Scenario: Non-existent file passthrough
- **WHEN** the agent runs `cat /nonexistent/path`
- **THEN** the cat plugin SHALL passthrough to bash instead of raising a Lua error

### Requirement: sift.fs.stat returns nil on error
The `sift.fs.stat()` function SHALL return `nil` instead of raising a Lua error when the path does not exist.

#### Scenario: Stat non-existent path
- **WHEN** `sift.fs.stat()` is called with a non-existent path
- **THEN** it SHALL return `nil`

#### Scenario: Stat existing path
- **WHEN** `sift.fs.stat()` is called with an existing path
- **THEN** it SHALL return a table with `size`, `is_dir`, and `is_file` fields

### Requirement: sift-read returns full content on change
The sift-read plugin SHALL return the full file content when a file has changed since the last read, instead of emitting a diff.

#### Scenario: File changed since last read
- **WHEN** a file is re-read and its content has changed
- **THEN** sift-read SHALL return the full new content with a one-line change notification

#### Scenario: File unchanged since last read
- **WHEN** a file is re-read and its content has not changed
- **THEN** sift-read SHALL return "unchanged" (current behavior preserved)

### Requirement: Cache invalidation on file write
The `sift.fs.write()` and `sift.fs.edit()` functions SHALL invalidate the path cache entry for the written file.

#### Scenario: Write invalidates cache
- **WHEN** `sift.fs.write()` writes to a file
- **THEN** the next `sift-read` of that file SHALL return the new content, not "unchanged"

#### Scenario: Edit invalidates cache
- **WHEN** `sift.fs.edit()` modifies a file
- **THEN** the next `sift-read` of that file SHALL return the new content, not "unchanged"

### Requirement: mtime-based cache staleness detection
The sift-read and cat plugins SHALL check the file's mtime before returning "unchanged". If the mtime is newer than the last read timestamp, the cache SHALL be considered stale and the file SHALL be re-read.

#### Scenario: File modified externally
- **WHEN** a file is modified by an external process between two sift-read calls
- **THEN** sift-read SHALL detect the mtime change and return the new content

#### Scenario: File unchanged
- **WHEN** a file's mtime has not changed since the last read
- **THEN** sift-read SHALL return "unchanged" if the content hash matches

### Requirement: command plugin has highest priority
The built-in `command` plugin SHALL have a higher priority than all user plugins, ensuring the `command` prefix always bypasses plugin interception.

#### Scenario: Command prefix bypass
- **WHEN** the agent runs `command cat <path>`
- **THEN** the command SHALL passthrough to bash regardless of other plugin patterns

#### Scenario: Plugin with same pattern
- **WHEN** a user plugin matches the same pattern as another plugin
- **THEN** the `command` prefix SHALL still bypass all plugin interception
