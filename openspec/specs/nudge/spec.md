## Purpose

Provide stronger, more actionable system prompt nudges to the agent when cached results are returned, reducing unnecessary re-reads.

## Requirements

### Requirement: System prompt nudge in sift.ts

The `before_agent_start` handler SHALL use stronger, more actionable instructions: tell the agent to say "same as before" and move on, tell the agent NOT to re-read or bypass unless file changed on disk, and remove vague "reuse it" language.

### Requirement: Unchanged message in all plugins

All plugins SHALL output a two-line format for unchanged messages, with the bypass command de-emphasized on the second line:

```
[sift] Justfile unchanged (cached)
      (bypass if stale: command cat /path/Justfile)
```

The primary line SHALL convey "unchanged (cached)" as the primary signal. The bypass command SHALL be on a secondary indented line so the agent does not treat "bypass" as a required action.

Affected plugins: sift-read, cat, sed, head, tail.