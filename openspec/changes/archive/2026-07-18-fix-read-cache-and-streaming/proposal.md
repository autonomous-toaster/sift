# Fix read cache, output duplication, redirect handling, and spawnHook quoting

## Why

Four bugs make sift unusable in practice:

1. **Read tool returns "unchanged" message as content** — `createReadTool`'s `readFile` calls `sift-read`, which returns a marker on cache hit. The agent gets `[sift] file unchanged` instead of content. Agent then tries workarounds (cp, python3, awk) that also break.

2. **Output duplication** — `exec_command()` writes chunks to stdout via `print!()`. Then `dispatch()` writes the returned output to stdout again via `print!("{output}")`. Every byte appears twice.

3. **Redirects broken** — `spawnHook` wraps commands with quotes, preventing shell from interpreting `<`/`>` as redirects. `sed -n '1,10p' < Justfile` crashes with `read /path/<: No such file or directory`.

4. **spawnHook quoting** — `shQuote` causes double-quoting bug when command already contains single-quoted paths.

5. **Agent doesn't understand sift markers** — The agent sees `[sift] ... unchanged` and `[sift] bypass: '...'` but doesn't know what to do. It tries workarounds (cp, python3, awk) instead of reusing cached content or following sift's bypass instructions.

## What Changes

### 1. Read tool: custom execute function

Replace `createReadTool` with custom `execute` function (same pattern as pi-readcache). The read tool calls `sift-read /path` and returns its output directly — marker, diff, or content. The agent is expected to understand the marker.

### 2. Output duplication: bash plugin stops returning `output`

The bash plugin returns `{ status = "handled", exit_code = 0 }` without the `output` field. `dispatch` already checks `if !output.is_empty()` before writing to stdout — empty output means nothing to write. The output is already streamed by `exec_command()`.

No new API field needed. Other plugins (sift-read, cat, sed, head, tail) continue returning `output` as before.

### 3. Redirect handling in `dispatch_full`

Parse `< file`, `> file`, `>> file` from args in `dispatch_full` (Rust), same place pipes are handled. Strip redirect args, pass file content as stdin / capture output. Complex redirects (`2>`, `&>`, heredocs) fall through to the shell.

### 4. spawnHook: `JSON.stringify` instead of `shQuote`

`JSON.stringify` is safe (no `/` escaping in Node.js). `$`/`` ` `` expansion inside double quotes is desired behavior. Redirects are now handled by `dispatch_full`, so no shell redirect interpretation needed.

### 5. System prompt nudge

Add a `before_agent_start` handler that appends a short nudge to the system prompt:

```
[sift] caches file reads. "[sift] ... unchanged" = content cached, reuse it. If you need fresh content, run sift's bypass command. Prefer sift over workarounds (cp, python3...) to save tokens.
```

The nudge is always appended with the same text, so the system prompt hash is stable — prompt caching is not invalidated.
