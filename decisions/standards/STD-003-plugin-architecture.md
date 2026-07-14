# STD-003 · Plugin Architecture

## Plugin format (Lua)

Every plugin is a Lua file that returns a table with:

```lua
return {
    -- Required: plugin name (used for debugging and metrics)
    name = "my-plugin",

    -- Optional: dispatch priority (default 0). Higher wins on tie.
    -- Built-in plugins use -1000. The command plugin uses 1000.
    priority = 0,

    -- Optional: command pattern for matching (defaults to name).
    -- Used for longest-prefix matching against the command name.
    pattern = "my-plugin",

    -- Required: execution function.
    -- ctx: table with session context (session_id, cmd_count, cwd, command)
    -- args: array of command arguments (excluding command name)
    -- stdin: piped input string, or nil if no pipe
    -- Returns: table with status, output, exit_code
    execute = function(ctx, args, stdin)
        -- ...
        return { status = "handled", output = "...", exit_code = 0 }
    end
}
```

## Plugin return values

The `execute` function must return a table with:

| Field | Type | Description |
|---|---|---|
| `status` | string | One of: `"handled"`, `"unchanged"`, `"passthrough"`, `"truncated"` |
| `output` | string (optional) | Output to emit. Required for handled, truncated. |
| `exit_code` | integer (optional) | Exit code. Default 0. |
| `message` | string (optional) | Short message for unchanged status. |
| `fingerprint` | string (optional) | Unique identifier for unchanged detection. |

### Status values

- **`"handled"`** — Plugin processed the command. `output` is emitted to stdout.
- **`"unchanged"`** — Output is identical to a previous invocation. `message` is emitted as a short marker (e.g., `[sift] file unchanged`).
- **`"passthrough"`** — Plugin did not handle this command. sift executes the real binary via `std::process::Command` with pipes.
- **`"truncated"`** — Plugin truncated the output. `output` contains the summary. A bypass notice with the full output path is appended.

## Plugin context (ctx)

The `ctx` table passed to `execute()` contains:

| Field | Type | Description |
|---|---|---|
| `session_id` | string | Current AI_SESSION value |
| `cmd_count` | integer | Command counter (monotonically increasing) |
| `cwd` | string | Current working directory |
| `command` | string | The command name (e.g., "git", "cat") |

## Registry and dispatch

The registry maps command patterns to plugins using longest-prefix matching, then highest priority:

1. Build candidate list: `[cmd]`, `[cmd, arg1]`, `[cmd, arg1, arg2]`, ...
2. For each candidate (longest first), find a matching plugin pattern.
3. If multiple plugins match the same pattern, highest priority wins.
4. If no plugin matches, the `__default__` plugin is used (bash.lua).
5. If no `__default__` exists, the command is executed directly via `std::process::Command`.

## Plugin responsibilities

Each plugin must:
- Accept all standard flags for the command it wraps (or return `"passthrough"` for unsupported flags).
- Produce byte-for-byte identical output to the real command for the same input.
- Never silently drop or alter data — only compress whitespace, strip ANSI, or apply lossless transformations.
- Use `sift.cache.has(ctx, key)` and `sift.cache.set(ctx, key)` for caching.
- Use `sift.exec(cmd)` to run commands, which returns `(stdout, stderr, exit_code)`.

## Built-in plugins

| Plugin | Pattern | Priority | Description |
|---|---|---|---|
| command.lua | `command` | 1000 | Bypass all plugins, run real binary |
| reset.lua | `reset` | 1000 | Clear sift cache for current session |
| cat.lua | `cat` | 0 | File read with content-based caching |
| git_status.lua | `git` | 0 | Git status with working-tree-clean detection |
| bash.lua | `__default__` | -1000 | Default fallback, runs command via sift.exec() |

## User plugins

User plugins are loaded from:
1. `~/.config/sift/plugins/*.lua`
2. Paths in `SIFT_PLUGINS` env var (colon-separated)

User plugins are loaded after built-in plugins. At the same declared priority, user plugins win over built-ins (loaded later = higher effective priority).
