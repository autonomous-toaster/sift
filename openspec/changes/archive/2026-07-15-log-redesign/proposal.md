## Why

`sift.log` uses a confusing callable-table hybrid (metatable `__call` + sub-fields) that is non-idiomatic in Lua. `sift.log.nudge` is semantically misplaced — nudges are action-oriented agent instructions, not log messages. The API is stabilizing and now is the time to fix these before external plugins depend on the current interface.

## What Changes

- **BREAKING**: `sift.log(ctx, level, msg)` replaced by `sift.log.{info,warn,error,debug}(ctx, msg)` — level methods instead of callable table.
- **BREAKING**: `sift.log.nudge(ctx, msg)` moved to top-level `sift.nudge(ctx, msg)` — no longer under `sift.log`.

## Capabilities

### New Capabilities
- `log-level-methods`: Level-specific methods under `sift.log` table (info, warn, error, debug).
- `nudge-top-level`: `sift.nudge` as a standalone function, independent of `sift.log`.

### Modified Capabilities
*(none — no existing specs are changing)*

## Impact

- **sift-core/src/lua/api.rs**: `register_log` rewritten to register level methods instead of callable table. New `register_nudge` method. `register_sift_table` updated to call `register_nudge`.
- **No plugin changes**: No existing plugins in the codebase use `sift.log` or `sift.log.nudge`, so zero migration cost internally.
- **Breaking change**: External/custom plugins using `sift.log` or `sift.log.nudge` will need updating.
