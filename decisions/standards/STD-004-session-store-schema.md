# STD-004 · Session Store Schema

## Database

SQLite via sqlx. Single database file at `~/.sift/sessions.db`. Shared across all sessions — session scoping is handled by the `session_id` column in the `sift_cache` table, not by separate database files.

## Schema

```sql
-- File metadata cache
-- Tracks file hashes for change detection.
-- No content stored — re-read from filesystem when needed.
CREATE TABLE file_cache (
    path       TEXT NOT NULL,       -- canonical absolute path
    hash       TEXT NOT NULL,       -- SHA256 of file content
    mtime      INTEGER NOT NULL,    -- file mtime at last read (unix ms)
    size       INTEGER NOT NULL,    -- file size in bytes
    last_read  INTEGER NOT NULL,    -- unix ms timestamp
    read_count INTEGER DEFAULT 0,
    PRIMARY KEY (path, hash)
);

-- Conversation cache
-- Tracks what information has been shown to the model.
-- Used to emit "unchanged" markers instead of repeating content.
CREATE TABLE conversation_cache (
    item_type          TEXT NOT NULL,  -- 'file_content', 'command_output'
    item_id            TEXT NOT NULL,  -- 'path:content_hash:flags_hash' or 'cmd:fingerprint'
    estimated_tokens   INTEGER,        -- approximate token count (informational only)
    commands_since_at_create INTEGER NOT NULL,  -- session.cmd_count at creation
    first_shown        INTEGER NOT NULL,      -- unix ms
    last_shown         INTEGER NOT NULL,      -- unix ms
    shown_count        INTEGER DEFAULT 0,
    re_requested       INTEGER DEFAULT 0,     -- model asked again after "unchanged"
    raw_bytes          INTEGER,               -- raw output bytes before filtering
    filtered_bytes     INTEGER,               -- filtered output bytes after plugin processing
    reduction_bps      INTEGER,               -- token reduction in basis points (1/100 of a percent)
    plugin_name        TEXT,                  -- name of the plugin that handled this command
    output_format      TEXT,                  -- format of the output (json, toon, text)
    PRIMARY KEY (item_type, item_id)
);

-- Content cache for sift plugins
-- Tracks whether a specific piece of content has been seen by a session.
-- Used by plugins to emit "unchanged" markers instead of repeating content.
-- Key is content-based (path:hash), session scoping is via session_id column.
CREATE TABLE sift_cache (
    key        TEXT NOT NULL,       -- "path:hash" — pure content identity
    session_id TEXT NOT NULL,       -- AI_SESSION value, for per-session scoping
    created_at INTEGER NOT NULL,    -- unix ms timestamp
    PRIMARY KEY (key, session_id)
);
```

## Access patterns

All DB access goes through `SessionStore` methods in `session.rs`. No raw SQL outside this module.

```rust
impl SessionStore {
    async fn open(db_path: &Path) -> Result<Self>;

    // File cache
    async fn get_file_cache(&self, path: &str) -> Result<Option<FileCacheEntry>>;
    async fn upsert_file_cache(&self, entry: &FileCacheEntry) -> Result<()>;

    // Conversation cache
    async fn get_conversation(&self, item_type: &str, item_id: &str) -> Result<Option<ConversationEntry>>;
    async fn record_conversation(&self, item_type: &str, item_id: &str, ...) -> Result<()>;
    async fn increment_re_requested(&self, item_type: &str, item_id: &str) -> Result<()>;

    // Sift cache (per-session content cache)
    async fn cache_has(&self, key: &str, session_id: &str) -> Result<bool>;
    async fn cache_set(&self, key: &str, session_id: &str) -> Result<()>;
    async fn cache_reset(&self, session_id: &str) -> Result<()>;
}
```

## Table responsibilities

| Table | Purpose | Lifecycle |
|---|---|---|
| `file_cache` | File metadata (hash, mtime, size) for change detection | Persistent, cleared on explicit request |
| `conversation_cache` | What was shown to the LLM, with rich metadata | Persistent, cleared on session end |
| `sift_cache` | Simple content membership per session | Per-session, cleared via `sift.cache.reset(ctx)` |

## Staleness heuristic

The `commands_since_at_create` field in `conversation_cache` enables a simple staleness check: if `session.cmd_count - entry.commands_since_at_create > STALENESS_THRESHOLD` (50), the cache entry is considered stale and the plugin shows full content instead of emitting "unchanged". This is a heuristic, not a guarantee — the model can always ask to re-read.

The `sift_cache` table has no staleness heuristic — entries live until explicitly reset via `sift.cache.reset(ctx)`.
