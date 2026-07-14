# STD-006 · sift_cache Table Schema

## Database

SQLite via sqlx. Single database file at `~/.sift/sessions.db`. Shared across all sessions — session scoping is handled by the `session_id` column, not by separate database files.

## Schema

```sql
-- Content cache for sift plugins.
-- Tracks whether a specific piece of content (identified by key)
-- has been seen by a specific session.
-- Used by plugins to emit "unchanged" markers instead of repeating content.
CREATE TABLE sift_cache (
    key        TEXT NOT NULL,       -- "path:hash" — pure content identity
    session_id TEXT NOT NULL,       -- AI_SESSION value, for per-session scoping
    created_at INTEGER NOT NULL,    -- unix ms timestamp
    PRIMARY KEY (key, session_id)
);
```

## Columns

| Column | Type | Description |
|---|---|---|
| `key` | TEXT | Content identity. Format: `path:hash` where hash is SHA256 of content. No session information is encoded in the key. |
| `session_id` | TEXT | The AI_SESSION that created this entry. Used for per-session scoping and reset. |
| `created_at` | INTEGER | Unix millisecond timestamp when the entry was created. |

## Access patterns

All DB access goes through `SessionStore` methods in `session.rs`. No raw SQL outside this module.

```rust
impl SessionStore {
    /// Check if a key exists for the given session.
    /// Returns true if an entry with this key AND session_id exists.
    async fn cache_has(&self, key: &str, session_id: &str) -> Result<bool>;

    /// Record a key for the given session.
    /// Creates an entry with the current timestamp.
    async fn cache_set(&self, key: &str, session_id: &str) -> Result<()>;

    /// Clear all cache entries for the given session.
    /// Does not affect entries from other sessions.
    async fn cache_reset(&self, session_id: &str) -> Result<()>;
}
```

## SQL queries

```sql
-- cache_has
SELECT 1 FROM sift_cache WHERE key = ?1 AND session_id = ?2

-- cache_set
INSERT INTO sift_cache (key, session_id, created_at) VALUES (?1, ?2, ?3)
ON CONFLICT(key, session_id) DO NOTHING

-- cache_reset
DELETE FROM sift_cache WHERE session_id = ?1
```

## Key format

Keys follow the pattern: `{identifier}:{hash}`

| Pattern | Example | Description |
|---|---|---|
| `path:sha256` | `/etc/hosts:abc123...` | File content cache. Path is canonical absolute path. Hash is SHA256 of file content. |

The key is constructed by the plugin, not by the cache layer. The cache layer treats the key as an opaque string.

## Session scoping

Session scoping is handled entirely by the `session_id` column and the `WHERE` clause. The key itself contains no session information. This means:

- **Same content, same session**: `cache_has("path:hash", "A")` → HIT (row exists)
- **Same content, different session**: `cache_has("path:hash", "B")` → MISS (no row for session B)
- **Session reset**: `cache_reset("A")` → deletes all rows where session_id = "A"

## Lifecycle

- Entries are created by `sift.cache.set(ctx, key)`.
- Entries are checked by `sift.cache.has(ctx, key)`.
- Entries are deleted by `sift.cache.reset(ctx)`.
- There is no TTL or staleness check — entries live until explicitly reset.
- The `sift_cache` table is independent of `file_cache` and `conversation_cache` — clearing one does not affect the others.
