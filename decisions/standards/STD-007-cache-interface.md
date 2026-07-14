# STD-007 · Plugin Cache Interface (sift.cache.*)

## Overview

The `sift.cache` namespace provides a per-session content cache for plugins. It is a **set** — plugins check membership (`has`) and add entries (`set`). There is no value associated with entries; the cache tracks only "has this content been seen by this session?"

## API

### sift.cache.has(ctx, key) → bool

Check if a cache entry exists for the current session.

```lua
-- Returns true if this session has already seen this content.
local cached = sift.cache.has(ctx, "/etc/hosts:abc123...")
```

| Param | Type | Description |
|---|---|---|
| `ctx` | table | Plugin context (passed to execute()). Provides session_id for scoping. |
| `key` | string | Opaque cache key. Convention: `path:sha256`. |

Returns `true` if an entry with this key exists for the current session, `false` otherwise.

### sift.cache.set(ctx, key)

Record a cache entry for the current session.

```lua
-- Record that this session has seen this content.
sift.cache.set(ctx, "/etc/hosts:abc123...")
```

| Param | Type | Description |
|---|---|---|
| `ctx` | table | Plugin context (passed to execute()). Provides session_id for scoping. |
| `key` | string | Opaque cache key. Convention: `path:sha256`. |

Returns nothing. If the entry already exists, it is a no-op (ON CONFLICT DO NOTHING).

### sift.cache.reset(ctx)

Clear all cache entries for the current session.

```lua
-- Clear all cached content for this session.
sift.cache.reset(ctx)
```

| Param | Type | Description |
|---|---|---|
| `ctx` | table | Plugin context (passed to execute()). Provides session_id for scoping. |

Returns nothing. Only entries belonging to this session are deleted. Other sessions are unaffected.

## Why ctx as first argument?

The `ctx` parameter provides the cache layer with session context without requiring the plugin to manage session IDs. The cache layer extracts `ctx.session_id` internally. This means:

- Plugins don't construct session-scoped keys — they use pure content keys.
- The cache layer handles session scoping transparently.
- If the scoping strategy changes (e.g., adding staleness based on ctx.cmd_count), no plugin code needs to change.

## Usage pattern

```lua
-- cat.lua — file read plugin
execute = function(ctx, args, stdin)
    -- ... resolve path, read content ...
    local hash = sift.hash.sha256(content)
    local key = path .. ":" .. hash

    if sift.cache.has(ctx, key) then
        return {
            status = "unchanged",
            message = "[sift] " .. args[1] .. " unchanged"
        }
    end

    sift.cache.set(ctx, key)
    return { status = "handled", output = content, exit_code = 0 }
end
```

## Implementation (Rust)

```rust
fn register_cache(&self, sift: &Table) -> Result<()> {
    let cache = self.lua.create_table()?;
    let store = self.store.clone();

    // sift.cache.has(ctx, key)
    let f_has = self.lua.create_function(move |_, (ctx, key): (Table, String)| {
        let session_id: String = ctx.get("session_id")?;
        if let Some(ref store) = store {
            futures::executor::block_on(store.cache_has(&key, &session_id))
                .map_err(|e| mlua::Error::external(e.to_string()))
        } else {
            Ok(false)
        }
    })?;
    cache.set("has", f_has)?;

    // sift.cache.set(ctx, key)
    let store2 = self.store.clone();
    let f_set = self.lua.create_function(move |_, (ctx, key): (Table, String)| {
        let session_id: String = ctx.get("session_id")?;
        if let Some(ref store) = store2 {
            futures::executor::block_on(store.cache_set(&key, &session_id))
                .map_err(|e| mlua::Error::external(e.to_string()))
        } else {
            Ok(())
        }
    })?;
    cache.set("set", f_set)?;

    // sift.cache.reset(ctx)
    let store3 = self.store.clone();
    let f_reset = self.lua.create_function(move |_, ctx: Table| {
        let session_id: String = ctx.get("session_id")?;
        if let Some(ref store) = store3 {
            futures::executor::block_on(store.cache_reset(&session_id))
                .map_err(|e| mlua::Error::external(e.to_string()))
        } else {
            Ok(())
        }
    })?;
    cache.set("reset", f_reset)?;

    sift.set("cache", cache)?;
    Ok(())
}
```
