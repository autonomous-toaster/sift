## Why

The current rtk.lua plugin uses a hardcoded list of command patterns (docker, kubectl, gh, etc.) that must be manually updated when rtk adds new subcommands. This is fragile and requires constant maintenance. The git_status plugin was removed for poor behavior, and we want a zero-maintenance rtk integration that automatically picks up new rtk subcommands.

## What Changes

- **BREAKING**: rtk.lua pattern changes from hardcoded list to `"*"` wildcard — catches all commands not handled by more specific plugins.
- **NEW**: `find_plugin()` in the Rust dispatch system gains `"*"` wildcard support — a plugin with pattern `"*"` matches any command candidate.
- **MODIFIED**: rtk.lua execute logic simplified — tries `rtk <command>`, falls through to bash on non-zero exit.
- **REMOVED**: Hardcoded pattern list in rtk.lua (docker, podman, kubectl, oc, gh, glab, curl, wget, npm, pnpm, pip, uv).

## Capabilities

### New Capabilities
- `wildcard-plugin`: Support for `"*"` wildcard pattern in plugin dispatch — any plugin with pattern `"*"` matches all unmatched commands.

### Modified Capabilities
- `rtk-plugin`: Pattern changes from explicit list to `"*"` wildcard. Execute logic simplified to try `rtk <command>` and passthrough on failure.

## Impact

- **sift-core/src/lua/api.rs**: One-line change in `find_plugin()` to treat `"*"` as wildcard.
- **plugins/rtk.lua**: Pattern changed to `"*"`, execute logic simplified.
- **No change to cat.lua**: Its pattern `"cat"` (len 3) beats `"*"` (len 1) via longest-pattern sorting.
- **No change to other plugins**: All have longer patterns than `"*"`, so they take priority.
