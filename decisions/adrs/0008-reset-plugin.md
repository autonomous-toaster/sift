---
status: proposed
date: 2026-07-14
---

# Reset Plugin for Per-Session Cache Clearing

## Context and Problem Statement

When an AI agent starts a new task or switches context, cached file reads from the previous task may be stale. The agent needs a way to clear the cache without restarting the session. Currently there is no mechanism to clear the sift cache — `SessionStore::clear_session()` wipes everything globally with no per-session granularity.

How should cache reset work?

## Considered Options

* **Lua API function only** — `sift.cache.reset(ctx)`. Callable from any plugin, but not directly from the shell.
* **Built-in plugin only** — A `reset.lua` plugin that matches the `reset` command. Callable from the shell, but not from other plugins.
* **Both** — `sift.cache.reset(ctx)` as the Lua API, and a `reset.lua` plugin that calls it. Accessible from both the shell and plugins.

## Decision Outcome

Chosen option: **Both** — `sift.cache.reset(ctx)` as the Lua API function, and a built-in `reset.lua` plugin that calls it, because:

- The shell needs a way to reset: `sift -c "reset"` or typing `reset` in REPL mode.
- Plugins may need to reset programmatically (e.g., after detecting a context switch).
- The plugin is a thin wrapper over the API — minimal code, maximum flexibility.
- The `command` builtin provides an escape hatch: `command reset` runs the real bash reset.

### Consequences

* Good, because cache reset is accessible from both the shell and plugins.
* Good, because the `command` builtin provides a bypass for the rare case where the real `reset` is needed.
* Good, because the plugin is minimal — ~10 lines of Lua.
* Good, because the output is token-optimized: `[sift] ok` instead of a verbose message.
* Bad, because `reset` shadows the bash `reset` command. Acceptable — `command reset` bypasses the plugin.

## Reset plugin

```lua
-- reset.lua — clear sift cache for current session
return {
    name = "reset",
    priority = 1000,
    pattern = "reset",
    execute = function(ctx, args, stdin)
        sift.cache.reset(ctx)
        return { status = "handled", output = "[sift] ok\n", exit_code = 0 }
    end
}
```

## Lua API

```lua
-- sift.cache.reset(ctx) — clear all cache entries for this session
-- Called by the reset plugin, also available for programmatic use
sift.cache.reset(ctx)
```
