## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `append_prompt` field to `PluginEntry` struct and `load_plugin_from_str()` |
| T1.2 | Add `--append-prompt` CLI flag to sift that collects and outputs prompts |
| T1.3 | Ship `append_prompt` for built-in plugins (cat, curl, rtk, sift-read) |
| T1.4 | Update pi extension to inject `sift --append-prompt` into system prompt |

## ADDED Requirements

### Requirement: Plugin SHALL support optional `append_prompt` field

T1.1 SHALL complete BEFORE T1.2 SHALL start.

T1.1 SHALL complete BEFORE T1.3 SHALL start.

#### Scenario: Plugin declares append_prompt

**WHEN** T1.1 completes
**THEN** a plugin SHALL be able to declare `append_prompt = "string"` in its return table
**AND** `load_plugin_from_str()` SHALL read the field and store it in `PluginEntry`

### Requirement: sift SHALL expose prompts via `--append-prompt` flag

T1.2 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: sift --append-prompt outputs prompts

**WHEN** T1.2 runs `sift --append-prompt`
**THEN** it SHALL output all non-empty `append_prompt` strings from loaded plugins, one per line
**AND** it SHALL exit without running any command

### Requirement: Built-in plugins SHALL ship with append_prompt

T1.3 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: cat declares append_prompt

**WHEN** T1.3 completes
**THEN** `cat.lua` SHALL declare `append_prompt` describing markdown compression and how to get raw content

#### Scenario: curl declares append_prompt

**WHEN** T1.3 completes
**THEN** `curl.lua` SHALL declare `append_prompt` describing response transformation and how to access saved raw content

#### Scenario: rtk declares append_prompt

**WHEN** T1.3 completes
**THEN** `rtk.lua` SHALL declare `append_prompt` describing output compression and how to get raw output via `command` prefix

### Requirement: Extension SHALL inject append_prompt into system prompt

T1.4 SHALL complete AFTER T1.2 SHALL complete.

#### Scenario: before_agent_start injects prompts

**WHEN** T1.4 handles `before_agent_start`
**THEN** it SHALL execute `sift --append-prompt` and append the output to the system prompt
