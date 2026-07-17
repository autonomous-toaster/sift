# System prompt nudge

## ADDED

### before_agent_start handler

- Add `before_agent_start` event handler in `integrations/pi/sift.ts`
- Appends a short nudge to the system prompt on every turn
- Nudge text is always the same → system prompt hash is stable → prompt caching not invalidated

Nudge text:

```
[sift] caches file reads. "[sift] ... unchanged" = content cached, reuse it. If you need fresh content, run sift's bypass command. Prefer sift over workarounds (cp, python3...) to save tokens.
```

## Verification

- First turn: system prompt includes nudge
- Subsequent turns: system prompt includes same nudge (hash stable)
- Agent reuses cached content instead of re-reading
- Agent follows bypass instructions when fresh content is needed
