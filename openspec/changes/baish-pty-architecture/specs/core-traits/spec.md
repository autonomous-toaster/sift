# Core Traits and Plugin System

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Restructure into multi-crate workspace |
| T1.2 | Define PluginContext, PluginResult, Plugin trait, priority registry |
| T1.3 | Define StreamFilter trait, implement PassthroughFilter |

## Requirements

### Requirement: Workspace precedes core traits

T1.1 SHALL complete BEFORE T1.2 SHALL run.

#### Scenario: Workspace structure ready

- **WHEN** T1.1 creates the workspace with baish-core, baish-filters, baish crates
- **THEN** T1.2 SHALL define PluginContext, PluginResult, and Plugin trait in baish-core.

### Requirement: Core traits precede filters

T1.2 SHALL complete BEFORE T1.3 SHALL run.

#### Scenario: StreamFilter depends on core types

- **WHEN** T1.2 defines the Plugin trait
- **THEN** T1.3 SHALL define StreamFilter in baish-filters.

### Requirement: Priority-based plugin resolution

ALWAYS T1.2 SHALL resolve plugins by longest-prefix match, then highest priority.

#### Scenario: Longer prefix wins

- **WHEN** "docker ps" and "docker" are both registered
- **THEN** T1.2 SHALL match "docker ps" for command "docker ps".

#### Scenario: Higher priority overrides

- **WHEN** "cat" is registered with priority -100 and priority 100
- **THEN** T1.2 SHALL select the priority 100 plugin.

### Requirement: StreamFilter processes lines

ALWAYS T1.3 SHALL process output line by line through feed_line().

#### Scenario: Lines are processed sequentially

- **WHEN** T1.3 receives three lines
- **THEN** feed_line() SHALL be called three times.
