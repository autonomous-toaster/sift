# Stronger system prompt nudge and message cleanup

## MODIFIED

### System prompt nudge in sift.ts

Replace the `before_agent_start` handler text with stronger, more actionable instructions:
- Tell the agent to say "same as before" and move on
- Tell the agent NOT to re-read or bypass unless file changed on disk
- Remove vague "reuse it" language

### Unchanged message in all plugins

Change from single-line format:
```
[sift] Justfile unchanged (cached). bypass if stale: command cat /path/Justfile
```

To two-line format (bypass de-emphasized):
```
[sift] Justfile unchanged (cached)
      (bypass if stale: command cat /path/Justfile)
```

Affected plugins: sift-read, cat, sed, head, tail.

## Verification

- Agent sees "unchanged (cached)" as primary signal → says "same as before"
- Agent can still find the bypass command if needed (secondary line)
- Agent does NOT treat "bypass" as a required action
