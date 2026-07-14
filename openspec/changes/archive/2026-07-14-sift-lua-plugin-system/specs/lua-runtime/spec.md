# Lua Runtime

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add mlua dependency and initialize Lua VM at startup |
| T1.2 | Register `sift.*` API functions in the Lua VM |
| T1.3 | Load built-in plugins from embedded Lua strings |

## Requirements

### Requirement: Lua VM initializes before plugin loading

T1.1 SHALL complete BEFORE T1.2 SHALL run.

#### Scenario: VM is ready for API registration

- **WHEN** T1.1 creates the Lua VM with mlua
- **THEN** T1.2 SHALL register the `sift.*` API table in the VM's global scope.

### Requirement: API registration precedes plugin loading

T1.2 SHALL complete BEFORE T1.3 SHALL run.

#### Scenario: Plugins can call sift.* functions

- **WHEN** T1.2 registers the `sift.*` API
- **THEN** T1.3 SHALL load built-in plugins that call `sift.*` functions.

### Requirement: Lua VM uses Lua 5.4

ALWAYS T1.1 SHALL configure mlua to use Lua 5.4.

#### Scenario: Lua version is correct

- **WHEN** T1.1 initializes the VM
- **THEN** the VM SHALL run Lua 5.4 bytecode.

### Requirement: VM is thread-safe

ALWAYS T1.1 SHALL enable the `send` feature on mlua.

#### Scenario: Plugins run on any thread

- **WHEN** a plugin executes on a non-main thread
- **THEN** the Lua VM SHALL NOT panic or produce data races.
