# Pipeline Optimization — Last-Command Plugin Dispatch

## Task Reference

| Task ID | Description |
|---------|-------------|
| T8.1 | Parse pipeline structure from classified command in dispatch |
| T8.2 | Run preceding pipeline segments in bash and pipe to last segment plugin |
| T8.3 | Fall through to bash for entire pipeline when last segment has no plugin |
| T8.4 | Update cat.lua to handle piped stdin content (cache by hash, return unchanged on repeat) |
| T8.5 | Update dispatch in agent_mode and repl_mode to use classifier for pipeline detection |

## ADDED Requirements

### Requirement: Dispatch uses classifier for pipeline detection

ALWAYS T8.5 SHALL use the classifier to detect pipe separators in the full command string.

#### Scenario: Pipeline detected

- **WHEN** T8.5 classifies `"echo abc | cat"`
- **THEN** T8.5 SHALL detect `is_piped=true` and extract segments.

### Requirement: Pipeline parsed before dispatch routing

T8.1 SHALL complete BEFORE T8.2 runs.

T8.1 SHALL complete BEFORE T8.3 runs.

#### Scenario: Plugin match found

- **WHEN** T8.1 finds a plugin matching the last pipeline segment
- **THEN** T8.2 SHALL run.

#### Scenario: No plugin match

- **WHEN** T8.1 finds no plugin matching the last pipeline segment
- **THEN** T8.3 SHALL run.

### Requirement: Plugin-matched pipeline dispatches to plugin

ALWAYS T8.2 SHALL execute preceding segments in bash and pass combined stdout as stdin to the matched plugin.

#### Scenario: Echo piped to cat

- **WHEN** T8.2 processes `"echo abc | cat"` with cat.lua matching
- **THEN** T8.2 SHALL run preceding segments in bash and dispatch cat.lua with captured stdout as stdin.

### Requirement: Unmatched pipeline runs in bash

ALWAYS T8.3 SHALL execute the entire pipeline in bash when no plugin matches the last segment.

#### Scenario: Grep has no plugin

- **WHEN** T8.3 processes `"cat some/file | grep abc"` with no plugin for grep
- **THEN** T8.3 SHALL run the full pipeline in bash.

### Requirement: Cat caches piped content

ALWAYS T8.4 SHALL hash piped stdin, cache via sift.cache.set, and return unchanged on repeat.

#### Scenario: First pipe caches

- **WHEN** T8.4 receives stdin from a pipe
- **THEN** T8.4 SHALL hash and cache the content, return as handled.

#### Scenario: Repeat pipe returns unchanged

- **WHEN** T8.4 receives identical stdin again
- **THEN** T8.4 SHALL return status unchanged with short message.
