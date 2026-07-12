# Cat Plugin with Lua Override Example

## Task Reference

| Task ID | Description |
|---------|-------------|
| T7.1 | Refactor CatPlugin to use PluginContext |
| T7.2 | Create example Lua cat plugin |

## Requirements

### Requirement: Rust CatPlugin precedes Lua example

T7.1 SHALL complete BEFORE T7.2 SHALL run.

#### Scenario: Rust plugin validates the API

- **WHEN** T7.1 refactors CatPlugin to use PluginContext
- **THEN** T7.2 SHALL document the same API for Lua plugins.

### Requirement: CatPlugin uses PluginContext

ALWAYS T7.1 SHALL access cache through PluginContext, not Session directly.

#### Scenario: Context-based cache access

- **WHEN** T7.1 executes a cat command
- **THEN** T7.1 SHALL use PluginContext to check the cache.

### Requirement: Lua example documents plugin API

ALWAYS T7.2 SHALL document PluginContext fields, PluginResult variants, and cache functions.

#### Scenario: API is documented

- **WHEN** T7.2 creates the Lua example
- **THEN** the example SHALL include comments explaining each API function.
