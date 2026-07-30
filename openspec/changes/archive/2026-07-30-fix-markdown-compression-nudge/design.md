## Context

Current flow when agent reads a `.md` file:

```
Agent reads file → cat plugin
  → reads file from disk
  → compresses via mdmin (SILENTLY)
  → returns compressed content
  → Agent sees compressed markdown
  → Agent constructs edit based on compressed content
  → Edit fails: oldText doesn't match uncompressed file on disk
```

Target flow:

```
Agent reads file → cat plugin
  → reads file from disk
  → compresses via mdmin
  → emits nudge: "[sift] raw: 'command cat <path>'"
  → returns compressed content
  → Agent sees compressed markdown + nudge
  → Agent knows to use "command cat" for raw content before editing
```

## Goals / Non-Goals

**Goals:**
1. When cat.lua compresses markdown, emit a nudge with `command cat <path>`
2. When sift-read.lua compresses content, emit a nudge with `command cat <path>`
3. Match the pattern already used by curl.lua for consistency

**Non-Goals:**
- Changing the compression behavior itself
- Adding nudges for non-compressed content

## Decisions

### Decision 1: Use `sift.nudge()` for consistency
The curl plugin already uses `sift.store()` + nudge pattern. For cat/sift-read, we use `sift.nudge()` directly since the raw content is already on disk (no need to store it separately).

### Decision 2: Nudge format matches curl plugin
Use the same format as curl: `"raw: 'command cat <path>'"` so the agent recognizes the pattern.

## Risks / Trade-offs

- **Risk**: The nudge adds ~50 tokens per read. Acceptable for the correctness gain.
- **Risk**: Agent might ignore the nudge. Mitigation: the nudge is visible in the output, and the agent learned to use `command cat` in the session log.
