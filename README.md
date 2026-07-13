# sift — AI-optimized shell proxy

**sift** is a shell proxy that intercepts commands, runs them through Lua plugins, and optimizes output for LLM consumption. It reduces token waste by caching repeated reads, summarizing verbose output, and transforming data into compact formats.

## Quick start

```bash
# Build
cargo build --release

# Run a command (agent mode)
./target/release/sift -c "cat foo.rs"

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
       │     ├── reads file via sift.fs.read()
       │     ├── checks cache via sift.hash.sha256()
       │     ├── cache hit → "[sift] foo.rs unchanged"
       │     └── cache miss → returns file content
       │
       └── bash.lua (default fallback) runs via PTY
             └── returns raw output as-is
```

## Lua plugin API

Plugins are Lua scripts that return a table with `name`, `priority`, `pattern`, and `execute` function.

```lua
-- ~/.config/sift/plugins/my_plugin.lua
return {
    name = "my-command",
    priority = 0,
    pattern = "my-command",
    execute = function(ctx, args, stdin)
        -- ctx: { cwd, cmd_count, session_id }
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
| `"unchanged"` | Output identical to previous invocation. Emits short marker. |
| `nil, error` | Plugin failed. Falls through to next matching plugin. |

### Built-in plugins

| Plugin | Priority | Description |
|--------|----------|-------------|
| `command.lua` | 1000 | Bypass mechanism — `command cat foo` runs real `cat` |
| `cat.lua` | 0 | Caches file reads, returns "unchanged" on cache hit |
| `git_status.lua` | 0 | Fingerprints output, returns "working tree clean" |
| `bash.lua` | -1000 | Default fallback — runs command via PTY |

### `sift.*` API reference

```
sift.exec(cmd)                    → output, exit_code
sift.log(level, msg)              -- "info"|"warn"|"error"|"debug"
sift.exit(code)                   -- exit process
sift.output(text)                 -- emit text to agent

sift.cache.get(key)               → value or nil
sift.cache.set(key, val)          -- set cached value
sift.cache.has(key)               → boolean

sift.hash.sha256(data)            → hex string
sift.hash.md5(data)               → hex string

sift.fs.read(path, {offset?, limit?})  → file content
sift.fs.write(path, content)      -- write file
sift.fs.edit(path, edits)         -- apply text replacements
sift.fs.stat(path)                → {size, is_dir, is_file}
sift.fs.exists(path)              → boolean

sift.json.encode(val)             → JSON string
sift.json.decode(str)             → Lua table
sift.toon.encode(val)             → TOON string (token-optimized)
sift.toon.decode(str)             → Lua table
sift.jq.query(data, filter)       → JSON result (requires `jaq` CLI)

sift.env.get(key)                 → value or nil
sift.env.set(key, val)            -- set environment variable

sift.classify(cmd)                → {kind, name, args, is_piped, is_compound}
sift.token_count(text)            → estimated token count

sift.meta.session_id              -- current session ID
sift.meta.cmd_count               -- command counter
sift.meta.cwd                     -- working directory
sift.meta.raw_bytes               -- raw output size (writable)
sift.meta.filtered_bytes          -- filtered output size (computed)
```

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

- [`cat.lua`](docs/examples/cat.lua) — File read caching with hash-based dedup
- [`cargo_test.lua`](docs/examples/cargo_test.lua) — Test output optimization with jq + TOON

Install examples by copying to `~/.config/sift/plugins/`:

```bash
cp docs/examples/cat.lua ~/.config/sift/plugins/
```

## User plugins

sift loads plugins from:
1. Built-in plugins (embedded in binary)
2. `~/.config/sift/plugins/*.lua`
3. `SIFT_PLUGINS` environment variable (colon-separated paths)

User plugins override built-ins at the same priority level.

## Project structure

```
sift/
├── sift-core/          # Core library: Lua runtime, session store, classifier
│   └── src/
│       ├── lua.rs      # Lua VM, sift.* API, plugin dispatch
│       ├── session.rs  # SQLite session store with token tracking
│       └── classifier.rs  # Command classification via brush-parser
├── sift/               # Main binary
│   ├── src/
│   │   ├── main.rs     # Entry point, agent/REPL modes
│   │   └── pty.rs      # PTY management (legacy)
│   └── plugins/        # Built-in Lua plugins
│       ├── bash.lua
│       ├── cat.lua
│       ├── command.lua
│       └── git_status.lua
├── docs/examples/      # Example user plugins
│   ├── cat.lua
│   └── cargo_test.lua
└── openspec/           # OpenSpec change management
```

## Requirements

- Rust 1.75+
- bash (at `/bin/bash`, `/usr/bin/bash`, or in PATH)
- Optional: `jaq` CLI for `sift.jq.query()` (`cargo install jaq`)

## License

MIT
