## Why

Plugins transform content (compress, filter, summarize) but the agent doesn't know this upfront. It discovers transformations only after seeing `[sift]` messages in output, and even then the naming is confusing — `[sift]` looks like infrastructure, not agent hints.

This change creates a clear communication channel from plugins to the agent:

1. **`append_prompt`**: Plugin declares its behavior upfront → injected into system prompt via `sift --append-prompt`
2. **`[nudge]` prefix**: Rename `[sift]` → `[nudge]` for all agent-facing messages, making intent clear
3. **Nudges for all transforming plugins**: Every plugin that transforms output tells the agent how to get the original content

## What Changes

### `append_prompt` field
Plugins can declare an optional `append_prompt` string. sift collects all prompts at startup and exposes them via `sift --append-prompt`. The pi extension injects them into the system prompt.

### `[sift]` → `[nudge]` rename
All agent-facing messages change prefix from `[sift]` to `[nudge]`. Internal sift logging (INFO/WARN/ERROR/DEBUG) keeps `[sift]`.

### Nudge types
Two types of nudges for getting original content:
- `[nudge] raw: 'command cat <path>'` — saved raw content on disk (curl, cat, sift-read)
- `[nudge] command: 'git status'` — re-run with `command` prefix to bypass plugin (rtk, jq, git)

## Capabilities

### New Capabilities
- `append-prompt`: Plugins declare behavior hints injected into agent system prompt
- `nudge-command`: New nudge type for plugins that transform output (rtk, jq, etc.)

### Modified Capabilities
- `nudge`: Rename prefix from `[sift]` to `[nudge]` for all agent-facing messages
- `pi-extension`: Inject `sift --append-prompt` output into system prompt
