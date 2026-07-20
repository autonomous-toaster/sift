# Deferred Optimizations

## High Impact (not yet tried)

- **Lazy API registration**: Defer registration of rarely-used API modules (gain, jq, diff, store, args) until first access. Each SiftLua::new() registers ~48 Lua functions — most are never called in typical usage. Use Lua proxy tables with __index metamethods.

- **Reuse Lua VM across tests**: The test suite creates 63 SiftLua instances. Each Lua::new() creates a fresh Lua state. If tests could share a VM (with reset between tests), this overhead disappears. Requires refactoring test helpers.

- **Thread pool for record_conversation**: Currently spawns a new thread for every DB write. A small thread pool (1-2 threads) with a channel would be much cheaper. Only matters when AI_SESSION is set.

## Medium Impact

- **Pre-allocate args table with known capacity**: mlua doesn't support create_table_with_capacity, but we could use a Lua chunk that creates the table with pre-allocated slots.

- **Avoid `i64::try_from(len)` in hot path**: Use `len.min(i64::MAX as usize) as i64` instead of try_from + unwrap_or.

- **Cache `find_real_bash` result**: Already done with LazyLock.

- **StdinReader optimization**: Currently uses Arc<Mutex<...>>. Since StdinReader is only used within a single thread, a simpler approach could work. But RefCell is not Send, and mlua is used with `send` feature.

## Low Impact / Speculative

- **Use `to_string_lossy` instead of `display().to_string()`**: Already done via cwd_str cache.

- **Avoid `format!()` in burst detection key**: Only called on "unchanged" status, which is rare.

- **Combine ctx and args into single Lua table**: Would change Lua API — not worth it.

- **Use Lua bytecode cache**: Cache compiled Lua chunks for API registration. Complex, marginal gain.
