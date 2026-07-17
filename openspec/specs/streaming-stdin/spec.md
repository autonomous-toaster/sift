## Purpose

Provide streaming stdin access to Lua plugins via a `StdinReader` userdata type, enabling incremental processing of large inputs without loading entire files into memory.

## Requirements

### Requirement: StdinReader Lua userdata

The system SHALL provide a `StdinReader` Lua userdata type that wraps a `Box<dyn Read + Send>` and exposes `readline()`, `read(n)`, and `lines()` methods.

#### Scenario: Read line by line

- **WHEN** StdinReader is defined with `readline()` method
- **THEN** calling `stdin:readline()` SHALL return the next line as a string, or nil at EOF

#### Scenario: Read N bytes

- **WHEN** StdinReader is defined with `read(n)` method
- **THEN** calling `stdin:read(4096)` SHALL return up to 4096 bytes as a string

#### Scenario: Line iterator

- **WHEN** StdinReader is defined with `lines()` method
- **THEN** `for line in stdin:lines() do end` SHALL iterate over all lines until EOF

### Requirement: dispatch() accepts StdinReader

The `dispatch()` function SHALL accept an `Option<StdinReader>` parameter alongside the existing `Option<&str>` for backward compatibility.

#### Scenario: StdinReader passed to plugin

- **WHEN** dispatch() is updated to accept StdinReader
- **THEN** the plugin's `stdin` parameter SHALL be a StdinReader userdata when a reader is provided

#### Scenario: String stdin still works

- **WHEN** dispatch() is updated to accept StdinReader
- **THEN** passing a string as stdin SHALL still work (backward compatible)

### Requirement: dispatch_full() creates StdinReader for < file

When `dispatch_full()` detects `< file` in args, it SHALL open the file and create a `StdinReader` wrapping `BufReader<File>`, instead of reading the entire file into memory.

#### Scenario: Small file redirect

- **WHEN** `< small.txt` is handled where small.txt is 10KB
- **THEN** the plugin SHALL receive a StdinReader backed by BufReader<File>

#### Scenario: Large file redirect

- **WHEN** `< huge.bin` is handled where huge.bin is 500MB
- **THEN** the system SHALL NOT read the entire file into memory (no OOM)

### Requirement: Pipeline handler creates StdinReader

The pipeline handler SHALL create a `StdinReader` wrapping `Cursor<String>` from collected preceding command output, instead of passing a raw string.

#### Scenario: Piped input

- **WHEN** `cmd1 | cmd2` is handled where cmd1 produces output
- **THEN** cmd2's plugin SHALL receive a StdinReader backed by Cursor<String>

### Requirement: Existing plugins use StdinReader API

All existing plugins that read stdin SHALL use the StdinReader API (`readline()`, `read()`, `lines()`) instead of treating stdin as a raw string.

#### Scenario: Plugin reads stdin via lines()

- **WHEN** a plugin is updated to use `stdin:lines()`
- **THEN** the plugin SHALL process input incrementally without loading it entirely into memory

### Requirement: Backward-compatible tostring()

The StdinReader SHALL support `tostring()` that reads the entire stream into a string, for backward compatibility with plugins that expect a raw string.

#### Scenario: tostring() reads full stream

- **WHEN** `tostring()` is implemented on StdinReader
- **THEN** `tostring(stdin)` SHALL return the full content as a string