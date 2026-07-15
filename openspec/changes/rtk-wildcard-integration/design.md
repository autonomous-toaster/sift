## Context

The current rtk.lua plugin uses a hardcoded list of command patterns that must be manually updated when rtk adds new subcommands. This is fragile and creates maintenance burden. The git_status plugin was removed for poor behavior, and we want a zero-maintenance rtk integration.

## Goals / Non-Goals

**Goals:**
- Replace hardcoded pattern list with `"*"` wildcard in rtk.lua
- Add `"*"` wildcard support to the Rust dispatch system
- Ensure cat.lua and other specific plugins still take priority over rtk

**Non-Goals:**
- No changes to cat.lua or other existing plugins
- No changes to the `__default__` fallback mechanism
- No changes to rtk's external behavior

## Decisions

### D1 — Wildcard pattern in find_plugin

A single-line change in `find_plugin()`: when checking if a plugin's pattern matches a candidate, also accept `"*"` as a wildcard.

```rust
// Before
e.patterns.iter().any(|p| p == candidate)

// After
e.patterns.iter().any(|p| p == candidate || p == "*")
```

**Rationale**: Minimal change, no new data structures, no special-casing in the dispatch loop. The existing longest-pattern sorting ensures specific plugins (cat, command, reset) beat the wildcard.

### D2 — rtk.lua uses wildcard

```lua
pattern = "*",
```

Execute tries `rtk <command>` and falls through on non-zero exit. No more pattern list to maintain.

**Rationale**: rtk returns in ~15-30ms for unknown commands, so the overhead of trying rtk for every unmatched command is negligible.

## Risks / Trade-offs

- **[rtk overhead for unmatched commands]** → ~15-30ms per call, negligible for a shell proxy. If rtk ever becomes slow, the passthrough mechanism ensures bash still works.
- **[rtk false positive]** → If rtk exits 0 for a command it shouldn't handle, the output goes through rtk instead of bash. Mitigation: rtk is designed to only handle commands it knows about; unknown commands return non-zero.
