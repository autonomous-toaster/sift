# Cat Plugin

## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Implement CatPlugin |
| T4.2 | Wire REPL loop in main.rs |

## Requirements

### Requirement: Cat plugin precedes REPL

T4.1 SHALL complete BEFORE T4.2 SHALL run.

#### Scenario: CatPlugin is registered before REPL starts

- **WHEN** T4.1 implements the CatPlugin
- **THEN** T4.2 SHALL register it in the PluginRegistry.

### Requirement: Cache check on every read

ALWAYS T4.1 SHALL check the conversation cache when reading a file.

#### Scenario: First read emits full content

- **WHEN** T4.1 reads a file for the first time
- **THEN** T4.1 SHALL emit full content AND record a cache entry.

#### Scenario: Repeated read emits unchanged marker

- **WHEN** T4.1 reads an unchanged file AND fewer than 50 commands have passed
- **THEN** T4.1 SHALL emit `[baish] <file> unchanged since last read`.

#### Scenario: Stale cache emits full content

- **WHEN** T4.1 reads an unchanged file AND 50 or more commands have passed
- **THEN** T4.1 SHALL emit full content.

### Requirement: Passthrough for unsupported flags

ALWAYS T4.1 SHALL return Passthrough when flags are present.

#### Scenario: Cat with flags falls through to real cat

- **WHEN** the input is `cat -n foo.rs`
- **THEN** T4.1 SHALL return Passthrough.

### Requirement: Re-request increments counter

ALWAYS T4.1 SHALL increment re_requested when the model requests the same file again following an Unchanged response.

#### Scenario: Model re-requests after unchanged

- **WHEN** T4.1 emitted Unchanged AND the same file is requested again
- **THEN** T4.1 SHALL emit full content on the next invocation.

### Requirement: Git plugin precedes registration

T6.1 SHALL complete BEFORE T6.2 SHALL run.

#### Scenario: Git plugin registered

- **WHEN** T6.1 implements the GitPlugin
- **THEN** T6.2 SHALL register it in the PluginRegistry.

### Requirement: Cargo plugin precedes registration

T7.1 SHALL complete BEFORE T7.2 SHALL run.

#### Scenario: Cargo plugin registered

- **WHEN** T7.1 implements the CargoPlugin
- **THEN** T7.2 SHALL register it in the PluginRegistry.
