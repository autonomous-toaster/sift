# sift — AI-optimized shell proxy

**sift** is a shell proxy that intercepts commands, runs them through Lua plugins, and optimizes output for LLM consumption. It reduces token waste by caching repeated reads, summarizing verbose output, and transforming data into compact formats.

## Quick start

```bash
# Build
cargo build --release

# Run a command (agent mode)
./target/release/sift -c "cat foo.rs"

# With session ID for token tracking
AI_SESSION=my-session ./target/release/sift -c "cat foo.rs"

# Interactive REPL
./target/release/sift --shell
```

## How it works

```
User runs: cat foo.rs
       │
       ▼
  sift parses and classifies the command
       │
       ├── cat.lua (plugin) intercepts "cat"
       │     ├── reads file via sift.fs.read(ctx, path)
       │     ├── checks cache via sift.hash.sha256(ctx, data)
       │     ├── cache hit → "[sift] foo.rs unchanged"
       │     └── cache miss → returns file content
       │
       └── bash.lua (default fallback) runs via PTY
             └── returns raw output as-is
```

## Lua plugin API

Plugins are Lua scripts that return a table with `name`, `priority`, `pattern`, and `execute` function.

```lua
-- plugins/my_plugin.lua or ~/.config/sift/plugins/my_plugin.lua
return {
    name = "my-command",
    priority = 0,
    pattern = "my-command",       -- string or string[] for multi-pattern
    execute = function(ctx, args, stdin)
        -- ctx: { cwd, cmd_count, session_id, command }
        -- args: command arguments (table)
        -- stdin: piped input (string or nil)
        return {
            status = "handled",
            output = "result",
            exit_code = 0
        }
    end
}
```

### Plugin return values

| Status | Description |
|--------|-------------|
| `"handled"` | Plugin consumed the command. `output` is sent to the agent. |
| `"passthrough"` | Run the real binary (used by `command` plugin). |
| `"unchanged"` | Output identical to previous invocation. Emits short marker + nudge. |
| `nil, error` | Plugin failed. Falls through to next matching plugin. |

### Built-in plugins

| Plugin | Priority | Pattern | Description |
|--------|----------|---------|-------------|
| `command.lua` | 1000 | `"command"` | Bypass mechanism — `command cat foo` runs real `cat` |
| `reset.lua` | 1000 | `"reset"` | Clear sift cache for current session |
| `cat.lua` | 0 | `"cat"` | Caches file reads + piped stdin, returns "unchanged" on cache hit |
| `git_status.lua` | 0 | `"git status"` | Fingerprints output, returns "working tree clean" |
| `bash.lua` | -1000 | `"__default__"` | Default fallback — runs command via bash |

### Shipped optional plugins (`plugins/`)

| Plugin | Pattern(s) | Description |
|--------|------------|-------------|
| `openspec.lua` | `"openspec"` | Injects `--json` flag, converts output via `sift.json.shortest()` |
| `rtk.lua` | `["docker", "podman", "kubectl", "oc", "gh", "glab", "curl", "wget", "npm", "pnpm", "pip", "uv"]` | Delegates matching commands to `rtk` binary |

### `sift.*` API reference

All functions take `ctx` as first argument for API consistency.

```
sift.exec(ctx, cmd)               → output, stderr, exit_code
sift.log(ctx, level, msg)         -- "info"|"warn"|"error"|"debug"
sift.log.nudge(ctx, msg)          -- accumulate nudge message
sift.exit(ctx, code)              -- exit process
sift.output(ctx, text)            -- emit text to agent

sift.cache.has(ctx, key)          → boolean
sift.cache.set(ctx, key)          -- set cached key
sift.cache.reset(ctx)             -- clear all cache for session

sift.hash.sha256(ctx, data)       → hex string
sift.hash.md5(ctx, data)          → hex string

sift.fs.read(ctx, path, {offset?, limit?})  → file content
sift.fs.write(ctx, path, content) -- write file
sift.fs.edit(ctx, path, edits)    -- apply text replacements
sift.fs.stat(ctx, path)           → {size, is_dir, is_file}
sift.fs.exists(ctx, path)         → boolean

sift.json.encode(ctx, val)        → JSON string
sift.json.decode(ctx, str)        → Lua table
sift.json.shortest(ctx, raw, formats)  → token-optimized JSON
sift.toon.encode(ctx, val)        → TOON string (token-optimized)
sift.toon.decode(ctx, str)        → Lua table
sift.jq.query(ctx, data, filter)  → JSON result

sift.store(ctx, content, slug)    → path (writes to /tmp/sift/<session>/, emits nudge)

sift.env.get(ctx, key)            → value or nil
sift.env.set(ctx, key, val)       -- set environment variable

sift.classify(ctx, cmd)           → {name, args, is_piped, is_compound}
sift.token_count(ctx, text)       → estimated token count

sift.meta.session_id              -- current session ID
sift.meta.cmd_count               -- command counter
sift.meta.cwd                     -- working directory
sift.meta.raw_bytes               -- raw output size (writable)
sift.meta.filtered_bytes          -- filtered output size (computed)
```

### `sift.json.shortest()` — Token-aware JSON optimization

Selects the most token-efficient JSON representation:

```lua
local formats = {
    json = { max_string_len = 80, max_array_items = 10, max_depth = 5, max_keys = 20 },
    toon = true
}
local output = sift.json.shortest(ctx, raw_json, formats)
```

- Tries raw (compacted), compacted JSON, and TOON formats
- Measures token cost including nudge overhead
- Selects the shortest representation
- Stores raw original to disk and emits nudge when non-raw wins

### Nudge system

Nudges tell the agent how to access original/unfiltered content:

- **Explicit**: `sift.log.nudge(ctx, "msg")` — accumulate during plugin execution
- **Auto on error**: `sift.exec()` non-zero exit → stores raw output, nudges path
- **Auto on unchanged**: plugin returns `status = "unchanged"` → nudges cached filename
- **Auto on json.shortest**: non-raw format wins → stores raw original, nudges path
- **Auto on store**: `sift.store()` → nudges stored file path

Nudges are appended to plugin output as `[sift] <msg>` lines at end of dispatch.

## Plugin loading order

sift loads plugins from these locations (later overrides earlier at same priority):

1. **Built-in** — `bash.lua`, `command.lua`, `reset.lua` (embedded in binary)
2. **`plugins/`** — shipped optional plugins (`cat.lua`, `git_status.lua`, `openspec.lua`, `rtk.lua`)
3. **`~/.config/sift/plugins/`** — user plugins
4. **`$SIFT_PLUGINS`** — colon-separated paths

## Multi-pattern plugins

Plugins can match multiple commands using an array of patterns:

```lua
return {
    name = "rtk",
    pattern = {"docker", "podman", "kubectl", "gh"},
    execute = function(ctx, args, stdin)
        -- matches any of: docker, podman, kubectl, gh
    end
}
```

Longest-prefix matching ensures specific patterns (e.g., `"git status"`) beat generic ones (e.g., `"git"`).

## Pipeline optimization

When the last command in a pipeline matches a plugin, sift runs preceding segments in bash and pipes output to the plugin:

```
echo abc | cat  →  runs "echo abc" in bash, pipes to cat.lua for caching
```

If the last command has no matching plugin, the entire pipeline runs in bash.

## cd dispatch

sift handles `cd <dir> && <command>` by peeling the cd prefix, changing directory, and dispatching the rest through plugins:

```
cd /x && docker ps  →  chdir /x, dispatch "docker ps" through rtk.lua
```

Supports recursive chains (`cd /x && cd /y && cmd`), `pushd`, `popd`, and semicolon separators.

## Bypass mechanism

Use the `command` builtin to bypass all plugins and run the real binary:

```bash
# Normal — goes through sift plugins
cat foo.rs

# Bypass — runs /bin/cat directly
command cat foo.rs
```

This reuses existing bash semantics — `command` already means "bypass shell-defined behavior." In sift, plugins ARE shell-defined behavior.

## Token reduction tracking

sift tracks token reduction per command and stores metrics in `~/.sift/sessions.db`:

```
raw_bytes: 15200
filtered_bytes: 420
reduction: 97.2%
plugin: cargo_test
```

## Example plugins

See `docs/examples/` for plugin examples:

- [`cat.lua`](docs/examples/cat.lua) — File read caching with hash-based dedup + piped stdin support
- [`cargo_test.lua`](docs/examples/cargo_test.lua) — Test output optimization with jq + TOON

Install examples by copying to `plugins/` or `~/.config/sift/plugins/`:

```bash
cp docs/examples/cat.lua plugins/
```

## Project structure

```
sift/
├── plugins/             # Shipped optional plugins (filesystem-loaded)
│   ├── cat.lua
│   ├── git_status.lua
│   ├── openspec.lua
│   └── rtk.lua
├── sift-core/           # Core library: Lua runtime, session store, classifier
│   └── src/
│       ├── lua/         # Lua VM, sift.* API, plugin dispatch
│       │   ├── mod.rs
│       │   └── api.rs
│       ├── session.rs   # SQLite session store with token tracking
│       └── classifier.rs  # Command classification via brush-parser
├── sift/                # Main binary
│   ├── src/
│   │   └── main.rs      # Entry point, agent/REPL modes
│   └── plugins/         # Core built-in plugins (embedded in binary)
│       ├── bash.lua
│       ├── command.lua
│       └── reset.lua
├── docs/examples/        # Example user plugins
│   ├── cat.lua
│   └── cargo_test.lua
└── openspec/            # OpenSpec change management
```

## Requirements

- Rust 1.75+
- bash (at `/bin/bash`, `/usr/bin/bash`, or in PATH)
- Optional: `jaq` CLI for `sift.jq.query()` (`cargo install jaq`)
- Optional: `rtk` CLI for `rtk.lua` plugin delegation

## License

MIT
