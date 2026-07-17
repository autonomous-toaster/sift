## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Register sift.str.split_lines() in Rust |
| T2.2 | Register sift.str.slice_text() in Rust |
| T2.3 | Register sift.str.is_sensitive() in Rust |
| T2.4 | Update head.lua to use sift.str.* |
| T2.5 | Update tail.lua to use sift.str.* |
| T2.6 | Update sed.lua to use sift.str.* |
| T2.7 | Update sift-read.lua to use sift.str.* |
| T2.8 | Update cat.lua to use sift.str.is_sensitive() |
| T2.9 | Remove duplicated split_lines/slice_text/is_sensitive from all plugins |

## ADDED Requirements

### Requirement: sift.str.split_lines()

The system SHALL register `sift.str.split_lines(text)` as a Rust function that splits text into lines, handling trailing newlines. Returns a Lua table of strings.

T2.1 SHALL complete BEFORE T2.4 SHALL run.

#### Scenario: Split with trailing newline

- **WHEN** T2.1 registers `sift.str.split_lines("a\nb\nc\n")`
- **THEN** it SHALL return `{"a", "b", "c", ""}` (4 elements, trailing empty string for final newline)

#### Scenario: Split without trailing newline

- **WHEN** T2.1 registers `sift.str.split_lines("a\nb\nc")`
- **THEN** it SHALL return `{"a", "b", "c"}` (3 elements)

### Requirement: sift.str.slice_text()

The system SHALL register `sift.str.slice_text(text, start, end)` as a Rust function that extracts a range of lines (1-indexed). Returns the joined string.

T2.2 SHALL complete BEFORE T2.4 SHALL run.

#### Scenario: Slice within bounds

- **WHEN** T2.2 registers `sift.str.slice_text("a\nb\nc\nd", 2, 3)`
- **THEN** it SHALL return `"b\nc"`

#### Scenario: Slice past end

- **WHEN** T2.2 registers `sift.str.slice_text("a\nb", 5, 10)`
- **THEN** it SHALL return `""` (empty string)

### Requirement: sift.str.is_sensitive()

The system SHALL register `sift.str.is_sensitive(path)` as a Rust function that checks if a path matches sensitive file patterns (`.env*`, `*.pem`, `*.key`, etc.).

T2.3 SHALL complete BEFORE T2.8 SHALL run.

#### Scenario: Sensitive path detected

- **WHEN** T2.3 registers `sift.str.is_sensitive("/path/to/.env.production")`
- **THEN** it SHALL return `true`

#### Scenario: Non-sensitive path

- **WHEN** T2.3 registers `sift.str.is_sensitive("/path/to/main.rs")`
- **THEN** it SHALL return `false`

### Requirement: Plugins use sift.str.* instead of local functions

All plugins SHALL use `sift.str.split_lines()`, `sift.str.slice_text()`, and `sift.str.is_sensitive()` instead of defining their own local copies.

T2.4, T2.5, T2.6, T2.7, T2.8 SHALL complete CONCURRENTLY. T2.9 SHALL complete AFTER all of T2.4-T2.8 SHALL complete.

#### Scenario: head.lua uses sift.str

- **WHEN** T2.4 updates head.lua
- **THEN** head.lua SHALL call `sift.str.split_lines()` and `sift.str.slice_text()` instead of local functions

#### Scenario: cat.lua uses sift.str.is_sensitive

- **WHEN** T2.8 updates cat.lua
- **THEN** cat.lua SHALL call `sift.str.is_sensitive()` instead of local `is_sensitive()`
