# Shell Execution

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Initialize Cargo workspace with dependencies |
| T1.2 | Implement parser.rs — brush-parser wrapper |
| T1.3 | Implement plugin.rs — Plugin trait and registry |
| T2.1 | Implement dispatcher.rs — dispatch decisions |
| T2.2 | Implement pipeline handling |
| T2.3 | Implement builtins (cd, export, unset, exit) |
| T4.2 | Wire REPL loop in main.rs |

## Requirements

### Requirement: Parser provides AST for dispatcher

T1.2 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Parser output feeds dispatcher

- **WHEN** T1.2 completes
- **THEN** T2.1 SHALL receive a valid brush-parser `Program` AST.

### Requirement: Workspace scaffold precedes parser

T1.1 SHALL complete BEFORE T1.2 SHALL run.

#### Scenario: Dependencies available for parser

- **WHEN** T1.1 initializes the workspace
- **THEN** brush-parser SHALL be available as a dependency.

### Requirement: Plugin registry precedes dispatcher

T1.3 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Registry available for dispatch

- **WHEN** T1.3 implements the PluginRegistry
- **THEN** T2.1 SHALL query the registry for matching plugins.

### Requirement: Dispatch decisions feed pipeline

T2.1 SHALL complete BEFORE T2.2 SHALL run.

#### Scenario: Non-interceptable command delegates to bash

- **WHEN** T2.1 determines a command is not interceptable
- **THEN** T2.2 SHALL delegate to `/bin/bash -c`.

### Requirement: Pipeline execution precedes REPL

T2.2 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: REPL uses pipeline execution

- **WHEN** T4.2 receives a multi-command pipeline
- **THEN** T2.2 SHALL execute all commands in the pipeline.

### Requirement: Interception restricted to PTY output

ALWAYS T2.1 SHALL invoke a plugin only when stdout goes to PTY.

#### Scenario: Piped command not intercepted

- **WHEN** the input is `cat foo | grep bar`
- **THEN** T2.1 SHALL NOT invoke the CatPlugin for `cat`.

### Requirement: Pipeline delegation for all-real commands

ALWAYS T2.2 SHALL delegate to bash when no plugin is involved.

#### Scenario: All-real pipeline

- **WHEN** the input is `cat foo | sort | uniq` AND no plugins are registered
- **THEN** T2.2 SHALL exec `/bin/bash -c "cat foo | sort | uniq"`.

### Requirement: Pipe setup for mixed pipelines

ALWAYS T2.2 SHALL set up os_pipe pipes when at least one command invokes a plugin.

#### Scenario: Mixed pipeline

- **WHEN** the input is `cat foo | grep bar` AND a GrepPlugin is registered
- **THEN** T2.2 SHALL create a pipe between real `cat` and the GrepPlugin.
