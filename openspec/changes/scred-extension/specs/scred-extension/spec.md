## ADDED Requirements

### Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `scred-redactor` git dependency and `scred` feature to `sift-core/Cargo.toml` |
| T2.1 | Add `PatternSelector` field to `RedactionStream` in scred source |
| T3.1 | Register `sift.ext.scred` module in `api_reg_ext.rs` |
| T3.2 | Implement `create_transform()` returning two Lua functions |
| T3.3 | Implement `redact()` one-shot convenience |
| T3.4 | Implement opts parsing for pattern selection |
| T4.1 | Create user bash plugin `shell.lua` with `__default__` pattern |
| T5.1 | Delete old `plugins/scred.lua` |
| T6.1 | Add unit tests for `sift.ext.scred` |
| T6.2 | Add integration test for bash plugin with scred |

---

### Requirement: scred-redactor dependency is added before extension code

T1.1 SHALL complete BEFORE T3.1 SHALL start.

#### Scenario: extension registration depends on scred-redactor

- **WHEN** T1.1 completes
- **THEN** T3.1 SHALL have access to `scred-redactor` types

---

### Requirement: RedactionStream supports pattern selection before extension uses it

T2.1 SHALL complete BEFORE T3.2 SHALL use `PatternSelector`.

#### Scenario: extension uses PatternSelector from RedactionStream

- **WHEN** T2.1 adds `PatternSelector` to `RedactionStream`
- **THEN** T3.2 SHALL pass the selector to `RedactionStream::new()`

---

### Requirement: extension functions are implemented in order

T3.1 SHALL complete BEFORE T3.2 SHALL start. T3.2 SHALL complete BEFORE T3.3 SHALL start. T3.3 SHALL complete BEFORE T3.4 SHALL start.

#### Scenario: create_transform is the primary API

- **WHEN** T3.1 registers the `sift.ext.scred` module
- **THEN** T3.2 SHALL add `create_transform()` to the module

#### Scenario: redact() depends on create_transform

- **WHEN** T3.2 implements `create_transform()`
- **THEN** T3.3 SHALL use it internally for one-shot redaction

#### Scenario: opts parsing is added last

- **WHEN** T3.3 implements `redact()`
- **THEN** T3.4 SHALL add `redact` field parsing to both `create_transform()` and `redact()`

---

### Requirement: bash plugin is created after extension

T3.2 SHALL complete BEFORE T4.1 SHALL use `sift.ext.scred.create_transform()`.

#### Scenario: plugin depends on extension API

- **WHEN** T3.2 implements `create_transform()`
- **THEN** T4.1 SHALL call it from the plugin's `execute()` function

---

### Requirement: old plugin is removed after replacement is ready

T4.1 SHALL complete BEFORE T5.1 SHALL delete `plugins/scred.lua`.

#### Scenario: old plugin is replaced by bash plugin

- **WHEN** T4.1 creates the bash plugin
- **THEN** T5.1 SHALL remove the old plugin

---

### Requirement: tests run after implementation

T3.4 SHALL complete BEFORE T6.1 SHALL start. T4.1 SHALL complete BEFORE T6.2 SHALL start.

#### Scenario: unit tests verify extension

- **WHEN** T3.4 completes all extension functions
- **THEN** T6.1 SHALL test `create_transform()` and `redact()` with known secrets

#### Scenario: integration test verifies bash plugin

- **WHEN** T4.1 creates the bash plugin
- **THEN** T6.2 SHALL test it with a command containing secrets
