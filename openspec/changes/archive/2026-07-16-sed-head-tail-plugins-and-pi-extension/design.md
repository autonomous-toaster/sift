## Context

The agent uses `sed -n`, `head`, and `tail` to read file ranges. These currently bypass sift's cache. Plugins for these commands share the same range-aware cache as `sift-read` and `cat`. A pi extension routes all bash and read calls through sift.

## Goals / Non-Goals

**Goals:**
- sed, head, tail plugins with range-aware caching
- Pi extension overriding read and intercepting bash
- AI_SESSION propagation on every sift call
- Cache reset on compaction events

**Non-Goals:**
- No changes to existing plugins (cat, sift-read)
- No changes to the Rust core

## Decisions

### D1 — sed plugin passthrough logic

Only intercept when:
1. `-n` flag is present
2. Expression matches `^<start>,<end>p$` or `^<start>p$` (range print)
3. A file path is provided (not stdin)

Otherwise passthrough.

### D2 — head/tail plugin logic

Parse `-n <count>` or `-<count>` flag. For `head`, range is [1, count]. For `tail`, read full file to get total lines, range is [total-count+1, total]. Passthrough for `-c` (byte count) and other flags.

### D3 — Pi extension

```typescript
export default function (pi: ExtensionAPI) {
  // Override read tool
  pi.registerTool({
    name: "read",
    async execute(params) {
      const cmd = buildSiftReadCmd(params);
      return execSift(cmd, sessionId);
    }
  });

  // Intercept bash
  pi.on("tool_call", (event) => {
    if (isToolCallEventType("bash", event)) {
      event.input.command = wrapWithSift(event.input.command);
    }
  });

  // Reset on compaction
  const reset = () => execSift("reset", sessionId);
  pi.on("session_compact", reset);
  pi.on("session_tree", reset);
  pi.on("session_fork", reset);
  pi.on("session_switch", reset);
  pi.on("session_shutdown", reset);
}
```

### D4 — AI_SESSION propagation

Every sift execution uses `env: { ...process.env, AI_SESSION: sessionId }`.

## Risks / Trade-offs

- **sed parsing**: Complex edge cases (multiple expressions, inline scripts). Passthrough on uncertainty.
- **head/tail with pipes**: Only intercept when reading a file, not stdin.
