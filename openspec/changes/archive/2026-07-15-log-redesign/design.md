## Context

`sift.log` is currently registered as a callable table via Lua metatable `__call`, accepting `(ctx, level, msg)`. It also has a sub-field `sift.log.nudge(ctx, msg)`. This dual nature (callable + namespace) is non-idiomatic in Lua and confusing for plugin authors. The nudge functionality is semantically unrelated to logging — nudges are action-oriented instructions for the agent, not log messages.

The API is stabilizing. No existing plugins in the codebase use `sift.log` or `sift.log.nudge`, so the migration cost is zero internally.

## Goals / Non-Goals

**Goals:**
- Replace callable `sift.log(ctx, level, msg)` with level methods `sift.log.{info,warn,error,debug}(ctx, msg)`
- Move `sift.log.nudge` to top-level `sift.nudge(ctx, msg)`
- Keep output format identical (`[sift] LEVEL: msg`) for each level

**Non-Goals:**
- No changes to log output format or destination
- No changes to nudge accumulation or dispatch behavior
- No changes to auto-nudge logic (exec error, unchanged, json.shortest, store)

## Decisions

### D1 — Level methods instead of callable table

`sift.log` becomes a plain table with four function fields. No metatable. This is the standard Lua pattern for namespaced functions.

```lua
-- Before
sift.log(ctx, "info", "msg")

-- After
sift.log.info(ctx, "msg")
sift.log.warn(ctx, "msg")
sift.log.error(ctx, "msg")
sift.log.debug(ctx, "msg")
```

**Rationale**: Simple, idiomatic, no metatable magic. Each level is a first-class function.

### D2 — sift.nudge as top-level

`sift.nudge(ctx, msg)` is registered directly on the `sift` table, alongside `sift.exec`, `sift.store`, etc.

**Rationale**: Nudges are a core sift concept, not a logging concern. Top-level placement signals their importance and makes them discoverable.

### D3 — register_nudge as separate method

A new `register_nudge` method in `SiftLua`, called from `register_sift_table`. The nudge closure captures `self.nudges.clone()` as before — no behavioral change.

**Rationale**: Clean separation of concerns. `register_log` no longer has nudge-related code.

## Risks / Trade-offs

- **[Breaking change for external plugins]** → No internal usage, and the API is explicitly stabilizing. External plugin authors will need to update `sift.log` and `sift.log.nudge` calls. The error message on misuse is clear (Lua nil/table error).
