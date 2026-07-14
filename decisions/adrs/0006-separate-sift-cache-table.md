---
status: proposed
date: 2026-07-14
---

# Separate sift_cache Table with Per-Session Scoping

## Context and Problem Statement

The current cache implementation stores "has this been seen?" entries in the `conversation_cache` table with `item_type = "sift_cache"` and `item_id = session_id:path:hash`. This conflates two concerns:

1. **Conversation tracking** — what was shown to the LLM, with rich metadata (token counts, show counts, re-requests).
2. **Content caching** — has this file content been seen before, simple key-value membership.

The cache key includes `session_id`, preventing cross-session cache hits for identical content. The cache has no per-session reset mechanism.

Should the cache use a separate table, and how should session scoping work?

## Considered Options

* **Reuse conversation_cache** — Add session_id column, use item_type='sift_cache'. Fewer tables but wider rows, complex PK, schema migration risk.
* **Separate sift_cache table** — Simple schema (key, session_id, created_at), independent lifecycle, no migration risk.
* **In-memory HashMap** — No persistence, lost on restart, no cross-session sharing.

## Decision Outcome

Chosen option: **Separate sift_cache table with per-session scoping**, because:

- The cache is a simple set (key membership), not a rich tracking store.
- A separate table has a minimal schema (3 columns vs 13 for conversation_cache).
- Independent lifecycle — cache can be cleared without affecting conversation history.
- Per-session scoping is handled by the cache layer (WHERE session_id = ?), not by encoding session_id into the key.
- The key is purely content-based (path:hash), enabling cross-session cache hits for identical content.

### Consequences

* Good, because the cache key is `path:hash` — pure content identity, no session encoding.
* Good, because per-session scoping is a WHERE clause, not key manipulation — the cache layer handles it transparently.
* Good, because `sift.cache.reset(ctx)` can delete entries for a single session without affecting others.
* Good, because the schema is trivially simple — key, session_id, created_at.
* Good, because the cache can be cleared independently of conversation history.
* Bad, because there's one more table to manage. Acceptable — the schema is minimal.

## Schema

```sql
CREATE TABLE sift_cache (
    key        TEXT NOT NULL,       -- "path:hash" — pure content identity
    session_id TEXT NOT NULL,       -- AI_SESSION value, for per-session scoping
    created_at INTEGER NOT NULL,    -- unix ms timestamp
    PRIMARY KEY (key, session_id)
);
```

## Access patterns

```rust
impl SessionStore {
    /// Check if a key exists for the given session.
    async fn cache_has(&self, key: &str, session_id: &str) -> Result<bool>;

    /// Record a key for the given session.
    async fn cache_set(&self, key: &str, session_id: &str) -> Result<()>;

    /// Clear all cache entries for the given session.
    async fn cache_reset(&self, session_id: &str) -> Result<()>;
}
```
