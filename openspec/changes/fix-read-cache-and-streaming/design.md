# Design

## 1. Read tool: custom execute function

**File**: `integrations/pi/sift.ts`

Replace `createReadTool` with a custom tool definition. The `execute` function:

1. Calls `siftExec("sift-read " + shQuote(path))` — sift resolves the path internally
2. Returns the output directly — marker, diff, or content
3. Handles image files by reading them directly (same as pi-readcache)

No `bypass_cache` param, no `--fresh` logic — exact same interface as default read tool.

```typescript
const readTool = {
    name: "read",
    label: "read",
    description: "...",
    parameters: Type.Object({
        path: Type.String({ description: "..." }),
        offset: Type.Optional(Type.Number({ description: "..." })),
        limit: Type.Optional(Type.Number({ description: "..." })),
    }),
    execute: async (_toolCallId, params, _signal, _onUpdate, ctx) => {
        const output = siftExec(`sift-read ${shQuote(params.path)}`);
        // Handle images...
        return { content: [{ type: "text", text: output }], details: {} };
    },
};
```

## 2. Output duplication: bash plugin stops returning `output`

**File**: `sift/plugins/bash.lua`

Remove the `output` field from the return value. The output is already streamed by `sift.exec()` → `exec_command()`.

```lua
-- Before
return {
    status = "handled",
    output = combined,
    exit_code = exit_code
}

-- After
return {
    status = "handled",
    exit_code = exit_code
}
```

**File**: `sift-core/src/lua/api.rs` — `dispatch()` function

No change needed. The existing code already handles missing `output`:
```rust
let output: String = result.get("output").unwrap_or_default();
// ...
if !output.is_empty() {
    print!("{output}");
    let _ = std::io::stdout().flush();
}
```

Empty `output` → nothing written to stdout. The streamed output from `exec_command()` is the only output.

## 3. Redirect handling in `dispatch_full`

**File**: `sift-core/src/lua/api.rs` — `dispatch_full()` function

Add redirect parsing after pipeline handling and before normal dispatch:

```rust
// Handle < file redirect
if let Some(pos) = args.iter().position(|a| a == "<") {
    if pos + 1 < args.len() {
        let file_path = &args[pos + 1];
        if let Ok(content) = std::fs::read_to_string(file_path) {
            // Remove < and file from args
            let mut clean_args = args.to_vec();
            clean_args.remove(pos);
            clean_args.remove(pos);
            return self.dispatch(name, &clean_args, Some(&content));
        }
    }
}

// Handle > file and >> file redirect
if let Some(pos) = args.iter().position(|a| a == ">" || a == ">>") {
    if pos + 1 < args.len() {
        let file_path = &args[pos + 1];
        let append = args[pos] == ">>";
        // Remove > and file from args
        let mut clean_args = args.to_vec();
        clean_args.remove(pos);
        clean_args.remove(pos);
        let (output, exit_code, plugin) = self.dispatch(name, &clean_args, stdin)?;
        if exit_code == 0 {
            if append {
                let _ = std::fs::OpenOptions::new()
                    .append(true).create(true).open(file_path)
                    .and_then(|mut f| std::io::Write::write_all(&mut f, output.as_bytes()));
            } else {
                let _ = std::fs::write(file_path, &output);
            }
        }
        return Ok((output, exit_code, plugin));
    }
}
```

Only handles `< file`, `> file`, `>> file`. Complex redirects (`2>`, `&>`, heredocs) fall through to the shell.

## 5. System prompt nudge

**File**: `integrations/pi/sift.ts`

Add a `before_agent_start` handler that appends a short nudge to the system prompt:

```typescript
pi.on("before_agent_start", async (event) => {
  return {
    systemPrompt:
      event.systemPrompt +
      '\n\n[sift] caches file reads. "[sift] ... unchanged" = content cached, reuse it. ' +
      "If you need fresh content, run sift's bypass command. " +
      "Prefer sift over workarounds (cp, python3...) to save tokens.",
  };
});
```

The nudge text is always the same, so the system prompt hash is stable across turns — prompt caching is not invalidated.
