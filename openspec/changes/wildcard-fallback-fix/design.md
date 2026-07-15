## Context

The `"*"` wildcard in `find_plugin()` is checked during the per-candidate loop alongside specific patterns. Since `"*"` matches any candidate, it can match a longer candidate (e.g., `"cat Cargo.toml"`) before a shorter candidate (`"cat"`) is checked, causing rtk to steal commands from more specific plugins.

## Goals / Non-Goals

**Goals:**
- Wildcard only matches when no specific pattern matches any candidate
- cat.lua correctly handles `cat Cargo.toml` (caching, unchanged detection)
- rtk still catches all unmatched commands

**Non-Goals:**
- No changes to plugin loading, sorting, or priority
- No changes to cat.lua or rtk.lua

## Decisions

### D1 — Two-pass matching in find_plugin

```rust
// Pass 1: specific patterns only
for candidate in candidates.iter().rev() {
    if let Some(entry) = self.plugins.iter().find(|e| e.patterns.iter().any(|p| p == candidate)) {
        return Some(entry);
    }
}

// Pass 2: wildcard fallback
self.plugins.iter().find(|e| e.patterns.iter().any(|p| p == "*"))
```

**Rationale**: Simple, minimal change. The two-pass approach ensures specific patterns always win, while wildcard still catches everything else.

## Risks / Trade-offs

- None — this is a pure bugfix that restores the intended behavior.
