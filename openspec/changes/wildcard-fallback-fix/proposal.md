## Why

The `"*"` wildcard pattern in `find_plugin()` matches any candidate during the per-candidate loop, including longer candidates that should be handled by more specific plugins. This causes rtk to steal commands from cat.lua (e.g., `cat Cargo.toml` matches `"*"` before `"cat"` is checked), breaking caching and other plugin-specific behavior.

## What Changes

- **FIX**: `find_plugin()` checks specific patterns first for all candidates, then falls back to `"*"` wildcard only if no specific match is found.

## Capabilities

### New Capabilities
- `wildcard-fallback`: Wildcard `"*"` only matches as a last resort, after all specific patterns have been checked against all candidates.

## Impact

- **sift-core/src/lua/api.rs**: `find_plugin()` split into two passes — specific patterns first, wildcard fallback second.
