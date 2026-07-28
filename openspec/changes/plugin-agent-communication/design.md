## Context

### Current state
Plugins communicate with the agent through `[sift]` messages in output. These are:
- Cache hit notifications: `[sift] file unchanged (cached)`
- Raw content hints: `[sift] raw: 'command cat ...'`
- Diff summaries: `[sift] N lines changed of M`
- Reset confirmations: `[sift] ok (cleared)`

The `[sift]` prefix is ambiguous — it could be infrastructure, errors, or hints. The agent has learned to treat them as hints, but the naming is misleading.

### Target state
Two complementary channels:

```
┌─────────────────────────────────────────────────────────────┐
│  SYSTEM PROMPT (once per session)                            │
│                                                              │
│  sift --append-prompt output:                                │
│  "HTTP responses are transformed (JSON compressed,           │
│   PDF→markdown). Follow [nudge] raw: hints to access        │
│   the saved response without re-issuing the request."         │
│  "Output from git, docker, ls is compressed by rtk.          │
│   Use [nudge] command: hints to get raw output."             │
│  "Markdown files may be compressed via mdmin.               │
│   Use [nudge] raw: hints to get the original content."       │
├─────────────────────────────────────────────────────────────┤
│  PER-COMMAND OUTPUT                                          │
│                                                              │
│  [nudge] file.md unchanged (cached)                          │
│  [nudge] raw: 'command cat /tmp/sift/...'                    │
│  [nudge] command: 'git status'                               │
│  [nudge] 14 lines changed of 197                             │
└─────────────────────────────────────────────────────────────┘
```

## Goals / Non-Goals

**Goals:**
1. Add optional `append_prompt` field to plugin registration
2. Collect prompts at startup, expose via `sift --append-prompt`
3. Rename `[sift]` → `[nudge]` for all agent-facing messages
4. Add `[nudge] command:` hints for rtk, jq, git plugins
5. Update pi extension to inject `sift --append-prompt` into system prompt
6. Ship `append_prompt` for built-in plugins (cat, curl, rtk, sift-read)

**Non-Goals:**
- Changing how plugins work internally
- Adding dynamic/function-based prompts
- Removing the existing hardcoded nudge in the pi extension

## Decisions

### Decision 1: `append_prompt` is a static string
Simple, predictable, no runtime overhead. The prompt is set at plugin load time and never changes.

### Decision 2: Simple concatenation with newlines
`sift --append-prompt` outputs each plugin's prompt on its own line. Empty prompts are skipped. No headers, no grouping.

### Decision 3: Two nudge types
- `[nudge] raw: 'command cat <path>'` — points to saved content on disk. Used by curl, cat, sift-read.
- `[nudge] command: 'git status'` — tells agent to re-run with `command` prefix. Used by rtk, jq, git.

### Decision 4: `[sift]` kept for internal logging
`sift-core` log messages (INFO, WARN, ERROR, DEBUG) keep the `[sift]` prefix. Only agent-facing messages change to `[nudge]`.

## Risks / Trade-offs

- **Risk**: Agent has learned to parse `[sift]` messages. Rename may cause temporary confusion. Mitigation: the rename is mechanical and the meaning is identical — only the prefix changes.
- **Risk**: `append_prompt` adds tokens to the system prompt. Mitigation: prompts are short (1-2 lines each), and only loaded plugins contribute.
