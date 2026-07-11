# STD-004 · Session Store Schema

## Database

SQLite via sqlx. One database file per session, stored at `.baish/session_<AI_SESSION_ID>.db` relative to the project root.

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
    PRIMARY KEY (item_type, item_id)
);
```

## Access patterns

All DB access goes through `SessionStore` methods in `session.rs`. No raw SQL outside this module.

```rust
impl SessionStore {
    async fn open(session_id: &str, cwd: &Path) -> Result<Self>;
    async fn get_file_cache(&self, path: &str) -> Result<Option<FileCacheEntry>>;
    async fn upsert_file_cache(&mut self, entry: &FileCacheEntry) -> Result<()>;
    async fn get_conversation(&self, item_type: &str, item_id: &str) -> Result<Option<ConversationEntry>>;
    async fn record_conversation(&mut self, item_type: &str, item_id: &str, tokens: Option<i32>) -> Result<()>;
    async fn increment_re_requested(&mut self, item_type: &str, item_id: &str) -> Result<()>;
}
```

## Staleness heuristic

The `commands_since_at_create` field enables a simple staleness check: if `session.cmd_count - entry.commands_since_at_create > STALENESS_THRESHOLD` (50), the cache entry is considered stale and the plugin shows full content instead of emitting "unchanged". This is a heuristic, not a guarantee — the model can always ask to re-read.
