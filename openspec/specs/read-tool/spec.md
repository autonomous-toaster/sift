## Purpose

Replace the default pi read tool with a custom tool that routes through sift for caching, marker-based output, and image/file handling.

## Requirements

### Requirement: Custom read tool replacing createReadTool

The system SHALL replace `createReadTool` with a custom tool definition with its own `execute` function. The `execute` function SHALL call `siftExec("sift-read " + shQuote(path))`, with sift resolving the path internally. No `bypass_cache` param, no `--fresh` logic — same interface as the default read tool. Image files SHALL be handled by reading directly (same as pi-readcache). The agent SHALL receive marker/diff/content from `sift-read` and is expected to understand it.

#### Scenario: First read returns content

- **WHEN** `read(path="Justfile")` is called
- **THEN** it SHALL return file content on first read

#### Scenario: Cache hit returns marker

- **WHEN** `read(path="Justfile")` is called again and file is unchanged
- **THEN** it SHALL return `[sift] file unchanged` on cache hit

#### Scenario: Sliced read

- **WHEN** `read(path="Justfile", offset=10, limit=5)` is called
- **THEN** it SHALL return sliced content or range marker