## Why

`sift.toon.encode()` and `sift.toon.decode()` currently only use `encode_default` / `decode_default` — the simplest toon-format path. The crate supports custom delimiters, indent control, strict mode, and no-coerce mode, but none of these are exposed to Lua plugins. Plugins that need pipe-delimited output, strict validation, or type-preserving decode have no way to configure it.

## What Changes

- Replace `sift.toon.encode(ctx, val)` with `sift.toon.encode(data, options?)` — accepts an optional options table with `delimiter` and `indent` fields
- Replace `sift.toon.decode(ctx, str)` with `sift.toon.decode(str, options?)` — accepts an optional options table with `strict` and `no_coerce` fields
- Remove the `ctx` parameter from both functions (they're pure, like `sift.str.*`)
- Keep the existing behavior as default when no options are passed

## Capabilities

### New Capabilities
- (none)

### Modified Capabilities
- `sift-api`: `sift.toon.encode` and `sift.toon.decode` signatures change — accept optional options table, drop `ctx` parameter

## Impact

- **sift-core/src/lua/api_reg_io.rs**: Rewrite `register_json_toon` — replace `encode_default`/`decode_default` with configurable `encode`/`decode` that parses options from Lua table
- **plugins/cat.lua**, **plugins/sift-read.lua**: Update any calls to `sift.toon.*` (if they exist — likely none)
- **sift-core/src/lua/tests_plugins.rs**: Update smoke test to verify new signatures
- No new dependencies — toon-format already in Cargo.toml