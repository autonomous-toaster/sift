# Pipeline Optimization

## Purpose

Optimize pipelines by dispatching the last command through plugins when a match is found, running preceding segments in bash.

## Requirements

### Requirement: Dispatch uses classifier for pipeline detection

The system SHALL use the classifier to detect pipe separators in the full command string.

#### Scenario: Pipeline detected

- **WHEN** the system classifies `"echo abc | cat"`
- **THEN** the system SHALL detect `is_piped=true` and extract segments.

### Requirement: Pipeline parsed before dispatch routing

Pipeline parsing SHALL complete before dispatch routing runs.

#### Scenario: Plugin match found

- **WHEN** the system finds a plugin matching the last pipeline segment
- **THEN** the system SHALL run preceding segments in bash and pipe to the matched plugin.

#### Scenario: No plugin match

- **WHEN** the system finds no plugin matching the last pipeline segment
- **THEN** the system SHALL run the entire pipeline in bash.

### Requirement: Plugin-matched pipeline dispatches to plugin

The system SHALL execute preceding segments in bash and pass combined stdout as stdin to the matched plugin.

#### Scenario: Echo piped to cat

- **WHEN** the system processes `"echo abc | cat"` with cat.lua matching
- **THEN** the system SHALL run preceding segments in bash and dispatch cat.lua with captured stdout as stdin.

### Requirement: Unmatched pipeline runs in bash

The system SHALL execute the entire pipeline in bash when no plugin matches the last segment.

#### Scenario: Grep has no plugin

- **WHEN** the system processes `"cat some/file | grep abc"` with no plugin for grep
- **THEN** the system SHALL run the full pipeline in bash.

### Requirement: Cat caches piped content

The system SHALL hash piped stdin, cache via sift.cache.set, and return unchanged on repeat.

#### Scenario: First pipe caches

- **WHEN** the system receives stdin from a pipe
- **THEN** the system SHALL hash and cache the content, return as handled.

#### Scenario: Repeat pipe returns unchanged

- **WHEN** the system receives identical stdin again
- **THEN** the system SHALL return status unchanged with short message.
