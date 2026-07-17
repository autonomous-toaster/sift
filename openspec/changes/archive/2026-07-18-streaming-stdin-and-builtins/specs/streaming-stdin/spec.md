## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Define StdinReader Lua userdata type in Rust |
| T1.2 | Update dispatch() to accept and pass StdinReader |
| T1.3 | Update dispatch_full() to create StdinReader for < file |
| T1.4 | Update pipeline handler to create StdinReader from collected output |
| T1.5 | Update existing plugins to use StdinReader API |
| T1.6 | Add backward-compat tostring() for small inputs |

## ADDED Requirements

### Requirement: StdinReader Lua userdata

The system SHALL provide a `StdinReader` Lua userdata type that wraps a `Box<dyn Read + Send>` and exposes `readline()`, `read(n)`, and `lines()` methods.

T1.1 SHALL complete BEFORE T1.2 SHALL run. T1.2 SHALL complete BEFORE T1.3 SHALL run. T1.3 SHALL complete BEFORE T1.4 SHALL run. T1.4 SHALL complete BEFORE T1.5 SHALL run.

#### Scenario: Read line by line

- **WHEN** T1.1 defines StdinReader with `readline()` method
- **THEN** calling `stdin:readline()` SHALL return the next line as a string, or nil at EOF

#### Scenario: Read N bytes

- **WHEN** T1.1 defines StdinReader with `read(n)` method
- **THEN** calling `stdin:read(4096)` SHALL return up to 4096 bytes as a string

#### Scenario: Line iterator

- **WHEN** T1.1 defines StdinReader with `lines()` method
- **THEN** `for line in stdin:lines() do end` SHALL iterate over all lines until EOF

### Requirement: dispatch() accepts StdinReader

The `dispatch()` function SHALL accept an `Option<StdinReader>` parameter alongside the existing `Option<&str>` for backward compatibility.

T1.2 SHALL complete BEFORE T1.3 SHALL run.

#### Scenario: StdinReader passed to plugin

- **WHEN** T1.2 updates dispatch() to accept StdinReader
- **THEN** the plugin's `stdin` parameter SHALL be a StdinReader userdata when a reader is provided

#### Scenario: String stdin still works

- **WHEN** T1.2 updates dispatch() to accept StdinReader
- **THEN** passing a string as stdin SHALL still work (backward compatible)

### Requirement: dispatch_full() creates StdinReader for < file

When `dispatch_full()` detects `< file` in args, it SHALL open the file and create a `StdinReader` wrapping `BufReader<File>`, instead of reading the entire file into memory.

T1.3 SHALL complete BEFORE T1.5 SHALL run.

#### Scenario: Small file redirect

- **WHEN** T1.3 handles `< small.txt` where small.txt is 10KB
- **THEN** the plugin SHALL receive a StdinReader backed by BufReader<File>

#### Scenario: Large file redirect

- **WHEN** T1.3 handles `< huge.bin` where huge.bin is 500MB
- **THEN** the system SHALL NOT read the entire file into memory (no OOM)

### Requirement: Pipeline handler creates StdinReader

The pipeline handler SHALL create a `StdinReader` wrapping `Cursor<String>` from collected preceding command output, instead of passing a raw string.

T1.4 SHALL complete BEFORE T1.5 SHALL run.

#### Scenario: Piped input

- **WHEN** T1.4 handles `cmd1 | cmd2` where cmd1 produces output
- **THEN** cmd2's plugin SHALL receive a StdinReader backed by Cursor<String>

### Requirement: Existing plugins use StdinReader API

All existing plugins that read stdin SHALL use the StdinReader API (`readline()`, `read()`, `lines()`) instead of treating stdin as a raw string.

T1.5 SHALL complete AFTER T1.4 SHALL complete.

#### Scenario: Plugin reads stdin via lines()

- **WHEN** T1.5 updates a plugin to use `stdin:lines()`
- **THEN** the plugin SHALL process input incrementally without loading it entirely into memory

### Requirement: Backward-compatible tostring()

The StdinReader SHALL support `tostring()` that reads the entire stream into a string, for backward compatibility with plugins that expect a raw string.

T1.6 SHALL complete AFTER T1.5 SHALL complete.

#### Scenario: tostring() reads full stream

- **WHEN** T1.6 implements `tostring()` on StdinReader
- **THEN** `tostring(stdin)` SHALL return the full content as a string
