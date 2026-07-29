## Context

Nudge messages are accumulated in a `Vec<String>` behind `Arc<Mutex<...>>` during dispatch, then collected and appended to the output in `finalize_dispatch()`. They come from two sources:

1. **Plugin nudges** — Lua plugins call `sift.nudge(ctx, msg)` with a string
2. **Rust nudges** — generated in `api_reg_io.rs` (JSON shortest, store) and `api_reg_cache.rs` (error save)

The current format varies by plugin and Rust location. The `[nudge]` prefix is added by `collect_nudges()` in `api.rs`.

## Goals / Non-Goals

**Goals:**
- All "unchanged" nudges use format: `<name> unchanged — already in your context. <command> to re-read.`
- All "compressed" nudges use format: `compressed output. raw: <command>`
- Binary document nudge drops feature installation instructions
- Burst warning uses stable format
- Remove "cached" and "bypass if stale" from all messages

**Non-Goals:**
- No changes to the nudge collection/dispatch mechanism
- No changes to `append_prompt` strings
- No changes to the `[nudge]` prefix format

## Decisions

### 1. Unchanged nudge format

```
<nickname> unchanged — already in your context. <command> to re-read.
```

Examples:
- `config.lua unchanged — already in your context. command cat config.lua to re-read.`
- `piped content unchanged — already in your context.`
- `foo.rs lines 1-10 unchanged — already in your context. command head -n 10 foo.rs to re-read.`

The "already in your context" tells the agent why it matters: the content is already available, no need to re-read unless something changed.

### 2. Compressed output nudge format

```
compressed output. raw: <command>
```

Examples:
- `compressed output. raw: command git status`
- `compressed output. raw: command cat /tmp/sift/...`

The `raw:` is now contextual — it reads as "here's the raw version" rather than a standalone label.

### 3. Binary document nudge format

```
<name> is a binary document. <command> to read it.
```

Example:
- `report.pdf is a binary document. command cat report.pdf to read it.`

No feature installation instructions. The agent only needs the immediate workaround.

### 4. Burst warning format

```
Result is stable — file hasn't changed on disk. Same output until it does.
```

### 5. Error save nudge format

```
error output saved. raw: command cat <path>
```

### 6. Store nudge format

```
output saved. raw: command cat <path>
```

## Nudge Message Map

| Location | Current | New |
|---|---|---|
| `plugins/cat.lua:26` | `piped content unchanged since last read` | `piped content unchanged — already in your context.` |
| `plugins/cat.lua:73` | `raw: command cat <path>` | `compressed output. raw: command cat <path>` |
| `plugins/cat.lua:84` | `<name> unchanged (cached)\n(bypass if stale: command cat <path>)` | `<name> unchanged — already in your context. command cat <path> to re-read.` |
| `plugins/cat.lua:95` | `piped content unchanged (cached)` | `piped content unchanged — already in your context.` |
| `plugins/head.lua:45,53` | `<name> lines 1-N unchanged (cached)\n(bypass if stale: command head -n N <path>)` | `<name> lines 1-N unchanged — already in your context. command head -n N <path> to re-read.` |
| `plugins/tail.lua:46,54` | `<name> lines N-M unchanged (cached)\n(bypass if stale: command tail -n <count> <path>)` | `<name> lines N-M unchanged — already in your context. command tail -n <count> <path> to re-read.` |
| `plugins/sed.lua:64,72` | `<name> lines N-M unchanged (cached)\n(bypass if stale: command sed -n 'N,Mp' <path>)` | `<name> lines N-M unchanged — already in your context. command sed -n 'N,Mp' <path> to re-read.` |
| `plugins/sift-read.lua:80` | `<name> unchanged (cached)\n(bypass if stale: sift-read --fresh <path>)` | `<name> unchanged — already in your context. sift-read --fresh <path> to re-read.` |
| `plugins/sift-read.lua:96,134` | `raw: sift-read --raw <path>` | `compressed output. raw: sift-read --raw <path>` |
| `plugins/sift-read.lua:113` | `<name> is a binary document... Install sift with --features xberg...\n(fallback: command cat <path>)` | `<name> is a binary document. command cat <path> to read it.` |
| `plugins/sift-read.lua:157,159` | `<name> line N unchanged (cached)\n(bypass if stale: sift-read --fresh <path> N)` | `<name> line N unchanged — already in your context. sift-read --fresh <path> N to re-read.` |
| `plugins/sift-read.lua:169` | `<name> unchanged (cached)\n(bypass if stale: sift-read --fresh <path>)` | `<name> unchanged — already in your context. sift-read --fresh <path> to re-read.` |
| `plugins/rtk.lua:24` | `raw: command <cmd>` | `compressed output. raw: command <cmd>` |
| `plugins/jq.lua:72,83` | `raw: command <cmd>` | `compressed output. raw: command <cmd>` |
| `api.rs:449` | `(this will keep returning the same result until the file changes on disk)` | `Result is stable — file hasn't changed on disk. Same output until it does.` |
| `api_reg_io.rs:326` | `raw: command cat <path>` | `output saved. raw: command cat <path>` |
| `api_reg_io.rs:550` | `raw: command cat <path>` | `output saved. raw: command cat <path>` |
| `api_reg_cache.rs:122` | `raw: command cat <path>` | `error output saved. raw: command cat <path>` |

## Risks / Trade-offs

- **Agent adaptation**: Existing agents may have learned the old format. New format requires re-learning.
- **Test updates**: ~10 test assertions need updating to match new strings.
- **No behavioral change**: The nudge mechanism itself is unchanged — only the message strings.
