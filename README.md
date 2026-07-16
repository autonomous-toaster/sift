# sift — AI-optimized shell proxy

**sift** is a shell proxy that intercepts commands, runs them through Lua plugins, and optimizes output for LLM consumption. It reduces token waste by caching repeated reads, summarizing verbose output, and transforming data into compact formats.

## Quick start

```bash
# Build
cargo build --release

# Run a command (agent mode)
./target/release/sift -c "cat foo.rs"

# With session ID for cross-invocation caching
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
       │     ├── sha256 hash → check file-based cache
       │     ├── cache hit → "[sift] foo.rs unchanged" + bypass nudge
       │     └── cache miss → store content by hash, return content
       │
       ├── sift-read.lua (plugin) intercepts "sift-read"
       │     ├── supports offset/limit for range reads
       │     ├── --fresh flag bypasses cache
       │     ├── on cache miss: loads old content, emits unified diff
       │     └── shares cache with cat.lua (cross-plugin dedup)
       │
       └── bash.lua (default fallback) runs via PTY
             └── output streams to stdout in real-time
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
| `bash.lua` | -1000 | `"__default__"` | Default fallback — runs command via bash |

### Shipped optional plugins (`plugins/`)

| Plugin | Pattern | Description |
|--------|---------|-------------|
| `sift-read.lua` | `"sift-read"` | File read with offset/limit, hash caching, unified diff on change, `--fresh` bypass |
| `cat.lua` | `"cat"` | File read caching (shares cache with sift-read) |
| `openspec.lua` | `"openspec"` | Injects `--json` flag, converts output via `sift.json.shortest()` |
| `rtk.lua` | `"*"` (wildcard) | Delegates unmatched commands to `rtk` binary |

### `sift.*` API reference

All functions take `ctx` as first argument for API consistency.

```
sift.exec(ctx, cmd, {transform?}) → output, stderr, exit_code
  -- transform: optional function(chunk) → string for streaming transforms
sift.log.{info,warn,error,debug}(ctx, msg)
sift.nudge(ctx, msg)              -- accumulate nudge message
sift.exit(ctx, code)              -- exit process
sift.output(ctx, text)            -- emit text to stdout

sift.cache.has(ctx, key)          → boolean (in-memory, per-invocation)
sift.cache.set(ctx, key)          -- set cached key
sift.cache.reset(ctx)             -- clear in-memory cache
sift.cache.has_file(ctx, hash)    → boolean (file-based, persists across invocations)
sift.cache.store_file(ctx, hash, content)  -- persist content + create cache marker
sift.cache.load_file(ctx, hash)   → string|nil (load content by hash)
sift.cache.set_path_hash(ctx, path, hash)  -- track path → last hash
sift.cache.get_path_hash(ctx, path)        → string|nil
sift.cache.cleanup(ctx, max_age_ms?)       -- prune expired entries + orphan objects
sift.cache.clear_all(ctx)         -- delete all cache markers and objects

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

sift.diff(ctx, old, new)          → unified diff string (via similar crate)

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

- **Explicit**: `sift.nudge(ctx, "msg")` — accumulate during plugin execution
- **Auto on error**: `sift.exec()` non-zero exit → stores raw output, nudges path
- **Auto on unchanged**: plugin returns `status = "unchanged"` → plugin emits own bypass nudge
- **Auto on json.shortest**: non-raw format wins → stores raw original, nudges path
- **Auto on store**: `sift.store()` → nudges stored file path

Nudges are appended to plugin output as `[sift] <msg>` lines at end of dispatch.

## sift-read plugin

The `sift-read` plugin provides hash-based file reading with range support and diff emission:

```bash
# First read — caches content
sift-read Cargo.toml

# Second read — "unchanged" with bypass nudge
sift-read Cargo.toml
# → [sift] Cargo.toml unchanged since last read
# → [sift] bypass: 'sift-read --fresh Cargo.toml'

# Range read
sift-read Cargo.toml 5 10
# → [sift] Cargo.toml lines 5-14 unchanged

# Bypass cache
sift-read --fresh Cargo.toml

# After file edit — emits unified diff
sift-read Cargo.toml
# → @@ -24,4 +24,5 @@
# →  ...
```

Shares cache with `cat` plugin — `cat file.txt` then `sift-read file.txt` detects "unchanged" and vice versa.

## Streaming output

All command output streams to stdout/stderr in real-time via background reader threads. Plugins can transform chunks in-flight:

```lua
-- Raw streaming (default)
local output = sift.exec(ctx, "docker ps")

-- Transformed streaming
local output = sift.exec(ctx, "cat file.txt", {
    transform = function(chunk)
        return string.upper(chunk)
    end
})
```

## Content-addressed cache

File content is stored by sha256 hash at `/tmp/sift/<session>/`:

```
/tmp/sift/<session>/
├── cache/          # Cache markers: <hash> → {"created_at": <ms>, "size": <bytes>}
├── objects/        # File content: sha256-<hash>.txt
└── paths/          # Path-to-hash mapping: <path_hash> → <content_hash>
```

- Persists across `sift -c` invocations within the same `AI_SESSION`
- Auto-prunes entries older than 24h on each invocation
- `reset` plugin clears all cache data
- Sensitive paths (`.env*`, `*.pem`, etc.) bypass caching

## Plugin loading order

sift loads plugins from these locations (later overrides earlier at same priority):

1. **Built-in** — `bash.lua`, `command.lua`, `reset.lua` (embedded in binary)
2. **`plugins/`** — shipped optional plugins (`cat.lua`, `sift-read.lua`, `openspec.lua`, `rtk.lua`)
3. **`~/.config/sift/plugins/`** — user plugins
4. **`$SIFT_PLUGINS`** — colon-separated paths

## Wildcard pattern

Plugins with `pattern = "*"` match any command not handled by a more specific plugin. Used by `rtk.lua` to delegate all unmatched commands to the `rtk` binary. Specific patterns (e.g., `"cat"`) always beat wildcard via longest-pattern sorting.

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

For sift-read, use `--fresh` flag:

```bash
sift-read --fresh foo.rs
```

## Project structure

```
sift/
├── plugins/             # Shipped optional plugins (filesystem-loaded)
│   ├── cat.lua
│   ├── sift-read.lua
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
