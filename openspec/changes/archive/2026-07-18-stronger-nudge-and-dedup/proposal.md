# Stronger nudge and dedup protection

## Why

The agent still doesn't understand the "unchanged (cached)" message. In a recent session, it ran `cat /path/file` 6 times getting the same cache response, then used `python3` as a workaround. The agent said "I wasted tokens fighting a cache that was working correctly."

Three issues:
1. **Nudge too weak** — "reuse it" is vague. Agent doesn't know HOW to reuse.
2. **"bypass" triggers action reflex** — Agent sees it as a required action, not optional.
3. **No dedup protection** — Agent runs the same command 6 times with no feedback that it's futile.

## What Changes

### 1. Stronger system prompt nudge

Change from:
```
[sift] caches file reads. "[sift] ... unchanged" = content cached, reuse it. If you need fresh content, run sift's bypass command. Prefer sift over workarounds (cp, python3...) to save tokens.
```

To:
```
[sift] caches file reads. When you see "[sift] ... unchanged (cached)", the content is already in this conversation — say "same as before" and move on. Do NOT re-read or bypass the cache unless you have a specific reason to believe the file changed on disk.
```

### 2. Keep bypass command but de-emphasize it

The bypass command is per-command (different for cat, sed, head, tail, sift-read). The system prompt nudge only mentions `sift-read --fresh`. Removing the bypass entirely would leave the agent without a way to bypass for non-sift-read commands.

Change from:
```
[sift] Justfile unchanged (cached). bypass if stale: command cat /path/Justfile
```

To two-line format where the second line reads as secondary info:
```
[sift] Justfile unchanged (cached)
      (bypass if stale: command cat /path/Justfile)
```

The indented parenthetical reads as an optional note, not a required action.

### 3. Dedup protection with time-based burst detection

Track recent unchanged responses with timestamps. If the same command+status repeats 3+ times within a 10-second window, append a stronger hint:
```
[sift] Justfile unchanged (cached)
      (bypass if stale: command cat /path/Justfile)
[sift] (this will keep returning the same result until the file changes on disk)
```

Sliding window of last 10 entries, each with a timestamp. Entries older than 10 seconds are pruned. This detects bursts (agent spamming the same command) without false positives from spaced-out reads across different turns.
