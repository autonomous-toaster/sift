# sift — AI-optimized shell proxy

**sift** is a shell proxy that intercepts commands, runs them through Lua plugins, and optimizes output for LLM consumption. Its primary goal is to avoid repeated reads of the same file across different tools (`cat`, `sift-read`, etc.) by caching file content by sha256 hash. It also reduces token waste by transforming verbose output (JSON, TOON) into compact formats.

Inspired by [pi-readcache](https://github.com/Gurpartap/pi-readcache) — a read cache for the pi coding agent that avoids re-reading unchanged files.

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

When an agent runs a command through sift, the command is classified and dispatched to the best matching plugin:

```
sift -c "cat foo.rs"
       │
       ▼
  classify: name="cat", args=["foo.rs"]
       │
       ▼
  find_plugin: pattern "cat" → cat.lua
       │
       ▼
  cat.lua.execute(ctx, ["foo.rs"], stdin)
       │
       ├── reads file via sift.fs.read(ctx, path)
       ├── sha256 hash → check file-based cache
       ├── cache hit → "[sift] foo.rs unchanged" + bypass nudge
       └── cache miss → store content by hash, return content
```

Other plugins follow the same pattern — classification, matching, execution:

## Cross-tool caching

The core innovation: `cat` and `sift-read` share the same file-based cache. If you read a file with `cat`, then read it again with `sift-read`, the second read detects the hash and returns "unchanged" — no re-reading from disk, no re-emitting content to the model.

```
cat Cargo.toml          → caches by sha256
sift-read Cargo.toml    → cache hit → "[sift] Cargo.toml unchanged"
```

This is the same principle as pi-readcache: avoid re-transmitting content the model has already seen.

## Token reduction

Beyond caching, sift reduces token consumption by:

- **JSON compaction**: truncates long strings, summarizes large arrays, limits depth/keys
- **TOON format**: Token-Oriented Object Notation — a compact, human-readable format for structured data (30-50% fewer tokens than JSON)
- **Smart format selection**: `sift.json.shortest()` tries raw, compacted JSON, and TOON, selects the most token-efficient representation
- **Unchanged detection**: emits a short nudge instead of re-emitting file content
- **Range reads**: `sift-read file 5 10` reads only the lines the model needs
- **Curl JSON optimization**: auto-detects JSON responses, compresses them, stores raw for re-read

## Behavioral plugins

Beyond optimization, sift can enforce agent behavior:

- **git-commit hook**: forbids `-n`/`--no-verify` on `git commit`, returns exit code 1 with nudge explaining hooks must run. When `-n` is absent, passthrough runs the command directly in bash (bypasses all plugins).
- **Curl JSON optimizer**: auto-detects JSON responses via `-w "%{content_type}"`, compresses with `sift.json.shortest()`, stores raw JSON for re-read. Respects `-v`/`--verbose` (full output) and `-w`/`--write-out` (passthrough). Always propagates curl exit code.

## Gain tracking

sift tracks token reduction per command and per session. Run `sift --gain` to see aggregate stats:

```
$ AI_SESSION=my-session sift --gain
sift gain
─────────────────────────────────────
  Commands:    47
  Raw:         1.2 MB
  Filtered:    340 KB
  Reduction:   71.7% (8,944 bps)
  Bypasses:    3
  ─────────────────────────────────────
  Per plugin:
    cat.lua             15 calls   82.3% reduction
    sift-read.lua       12 calls   65.1% reduction
    bash.lua            10 calls    0.0% reduction
    command.lua          3 calls   (bypass)
  ─────────────────────────────────────
  Session: my-session
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
        -- ctx: { cwd, cmd_count, session_id, command, merge_stderr }
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

### Plugin return fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | `"handled"`, `"passthrough"`, or `"unchanged"` |
| `output` | string | Output to send to the agent |
| `exit_code` | integer | Process exit code |
| `raw_bytes` | integer (optional) | Size of raw output before transformation (for gain tracking) |
| `streamed` | boolean (optional) | If true, output was already streamed to stdout; dispatch won't print again |

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
| `head.lua` | `"head"` | First N lines of a file with caching |
| `tail.lua` | `"tail"` | Last N lines of a file with caching |
| `sed.lua` | `"sed"` | Line range extraction with caching |
| `curl.lua` | `"curl"` | JSON response optimizer — detects JSON, compresses via TOON, stores raw for re-read |
| `git-commit.lua` | `"git commit"` | Forbids `-n`/`--no-verify` on git commit, returns exit 1 + nudge. Passthrough runs directly in bash (not via rtk). |
| `openspec.lua` | `"openspec"` | Injects `--json` flag, converts output via `sift.json.shortest()` |
| `rtk.lua` | `"*"` (wildcard) | Delegates unmatched commands to `rtk` binary |

### `sift.*` API reference

All functions take `ctx` as first argument for API consistency. The `ctx` table has the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `cwd` | string | Current working directory |
| `cmd_count` | integer | Command counter for this session |
| `session_id` | string | Current session ID |
| `command` | string | The original command string |
| `merge_stderr` | boolean | Whether stderr should be merged (from `2>&1` redirect) |

```
sift.exec(ctx, cmd, {transform?, silent?, merge_stderr?}) → output, stderr, exit_code
  -- transform: optional function(chunk) → string for streaming transforms
  -- silent: if true, suppress stdout printing (for plugins that return output)
  -- merge_stderr: if true, keep streams separate, transform stdout only, append stderr raw
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
sift.toon.encode(data, options?)  → TOON string (pure, no ctx)
  -- options.delimiter: "comma" | "pipe"
  -- options.indent: "space2" | "space4"
sift.toon.decode(str, options?)   → Lua table (pure, no ctx)
  -- options.strict: true | false
  -- options.no_coerce: true | false
sift.jq.query(ctx, data, filter)  → JSON result

sift.diff(ctx, old, new)          → unified diff string (via similar crate)

sift.store(ctx, content, slug)    → path (writes to /tmp/sift/<session>/, emits nudge)

sift.env.get(ctx, key)            → value or nil
sift.env.set(ctx, key, val)       -- set environment variable

sift.classify(ctx, cmd)           → {name, args, is_piped, is_compound}
sift.token_count(ctx, text)       → estimated token count

sift.str.split_lines(text)        → {line1, line2, ...}  (pure, no ctx)
sift.str.slice_text(text, start, end) → string (pure, no ctx)
sift.str.is_sensitive(path)        → boolean (pure, no ctx)

sift.gain.report(flags?)          → gain report string
  -- flags.verbose: true | false
  -- flags.json: true | false
  -- flags.all: true | false (all sessions)
  -- flags.session: "session-id"
  -- flags.since: timestamp_ms

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

## curl plugin

The `curl` plugin auto-detects JSON responses and compresses them:

```bash
# JSON response — compressed via TOON, raw stored for re-read
curl https://jsonplaceholder.typicode.com/posts
# → [100]{userId,id,title,body}: ...

# Verbose requested — full output, no compression
curl -v https://api.example.com/data

# Custom -w format — passthrough, no interference
curl -w "\n%{http_code}" https://api.example.com/data

# Non-JSON response — body returned as-is
curl https://example.com/page.html
```

## git-commit plugin

The `git-commit` plugin prevents accidental hook bypass:

```bash
# Forbidden — returns exit 1 with nudge
git commit -m "fix" -n
# → [sift] git commit --no-verify (-n) is forbidden: hooks must run

# Allowed — passthrough runs directly in bash
git commit -m "fix"

# Other git commands — don't match "git commit" pattern,
# fall through to wildcard (rtk.lua) or default (bash.lua)
git status
git push
```

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
2. **`plugins/`** — shipped optional plugins (`cat.lua`, `sift-read.lua`, `head.lua`, `tail.lua`, `sed.lua`, `curl.lua`, `git-commit.lua`, `openspec.lua`, `rtk.lua`)
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
│   ├── curl.lua
│   ├── git-commit.lua
│   ├── head.lua
│   ├── openspec.lua
│   ├── rtk.lua
│   ├── sed.lua
│   ├── sift-read.lua
│   └── tail.lua
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