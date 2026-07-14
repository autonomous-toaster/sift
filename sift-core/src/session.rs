//! Session state and SQLite-backed session store.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

/// In-memory session state.
pub struct Session {
    /// Current working directory.
    pub cwd: PathBuf,
    /// Environment variables.
    pub env: HashMap<String, String>,
    /// Command counter (for staleness heuristic).
    pub cmd_count: u64,
    /// Session identifier.
    pub session_id: Option<String>,
    /// Optional SQLite-backed session store.
    pub store: Option<SessionStore>,
}

impl Session {
    /// Create a new session from the environment.
    #[must_use]
    pub fn from_env() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let env: HashMap<String, String> = std::env::vars().collect();
        let session_id = std::env::var("AI_SESSION").ok();

        Self {
            cwd,
            env,
            cmd_count: 0,
            session_id,
            store: None,
        }
    }

    /// Open the session store at `~/.sift/sessions.db`.
    pub async fn open_store(&mut self) {
        let db_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sift")
            .join("sessions.db");
        match SessionStore::open(&db_path).await {
            Ok(store) => {
                self.store = Some(store);
            }
            Err(e) => {
                eprintln!("sift: failed to open session store: {e}");
            }
        }
    }
}

/// A file cache entry.
#[derive(Debug, Clone)]
pub struct FileCacheEntry {
    /// Canonical absolute path.
    pub path: String,
    /// SHA256 hash of file content.
    pub hash: String,
    /// File mtime at last read (unix ms).
    pub mtime: i64,
    /// File size in bytes.
    pub size: i64,
    /// Timestamp of last read (unix ms).
    pub last_read: i64,
    /// Number of times read.
    pub read_count: i32,
}

/// A conversation cache entry with token tracking metrics.
#[derive(Debug, Clone)]
pub struct ConversationEntry {
    /// Type of item (`file_content`, `command_output`).
    pub item_type: String,
    /// Unique ID for the item.
    pub item_id: String,
    /// Estimated token count (informational).
    pub estimated_tokens: Option<i32>,
    /// Session `cmd_count` at creation.
    pub commands_since_at_create: i64,
    /// Timestamp of first show (unix ms).
    pub first_shown: i64,
    /// Timestamp of last show (unix ms).
    pub last_shown: i64,
    /// Number of times shown.
    pub shown_count: i32,
    /// Number of times model re-requested after "unchanged".
    pub re_requested: i32,
    /// Raw output bytes before filtering.
    pub raw_bytes: Option<i64>,
    /// Filtered output bytes after plugin processing.
    pub filtered_bytes: Option<i64>,
    /// Token reduction in basis points (1/100 of a percent).
    pub reduction_bps: Option<i64>,
    /// Name of the plugin that handled this command.
    pub plugin_name: Option<String>,
    /// Format of the output (json, toon, text).
    pub output_format: Option<String>,
}

/// SQLite-backed session store.
pub struct SessionStore {
    pool: SqlitePool,
}

impl SessionStore {
    /// Open or create a session store at the given path.
    pub async fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let opts = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS file_cache (
                path       TEXT NOT NULL,
                hash       TEXT NOT NULL,
                mtime      INTEGER NOT NULL,
                size       INTEGER NOT NULL,
                last_read  INTEGER NOT NULL,
                read_count INTEGER DEFAULT 0,
                PRIMARY KEY (path, hash)
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS conversation_cache (
                item_type          TEXT NOT NULL,
                item_id            TEXT NOT NULL,
                estimated_tokens   INTEGER,
                commands_since_at_create INTEGER NOT NULL,
                first_shown        INTEGER NOT NULL,
                last_shown         INTEGER NOT NULL,
                shown_count        INTEGER DEFAULT 0,
                re_requested       INTEGER DEFAULT 0,
                raw_bytes          INTEGER,
                filtered_bytes     INTEGER,
                reduction_bps      INTEGER,
                plugin_name        TEXT,
                output_format      TEXT,
                PRIMARY KEY (item_type, item_id)
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sift_cache (
                key        TEXT NOT NULL,
                session_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (key, session_id)
            )",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Get a file cache entry by path.
    pub async fn get_file_cache(&self, path: &str) -> Result<Option<FileCacheEntry>> {
        let row = sqlx::query_as::<_, (String, String, i64, i64, i64, i32)>(
            "SELECT path, hash, mtime, size, last_read, read_count FROM file_cache WHERE path = ?1",
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(path, hash, mtime, size, last_read, read_count)| FileCacheEntry {
            path,
            hash,
            mtime,
            size,
            last_read,
            read_count,
        }))
    }

    /// Upsert a file cache entry.
    pub async fn upsert_file_cache(&self, entry: &FileCacheEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO file_cache (path, hash, mtime, size, last_read, read_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(path, hash) DO UPDATE SET
                mtime = excluded.mtime,
                size = excluded.size,
                last_read = excluded.last_read,
                read_count = read_count + 1",
        )
        .bind(&entry.path)
        .bind(&entry.hash)
        .bind(entry.mtime)
        .bind(entry.size)
        .bind(entry.last_read)
        .bind(entry.read_count)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a conversation cache entry.
    pub async fn get_conversation(
        &self,
        item_type: &str,
        item_id: &str,
    ) -> Result<Option<ConversationEntry>> {
        let row = sqlx::query_as::<_, (String, String, Option<i32>, i64, i64, i64, i32, i32, Option<i64>, Option<i64>, Option<i64>, Option<String>, Option<String>)>(
            "SELECT item_type, item_id, estimated_tokens, commands_since_at_create,
                    first_shown, last_shown, shown_count, re_requested,
                    raw_bytes, filtered_bytes, reduction_bps, plugin_name, output_format
             FROM conversation_cache
             WHERE item_type = ?1 AND item_id = ?2",
        )
        .bind(item_type)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(item_type, item_id, estimated_tokens, commands_since_at_create, first_shown, last_shown, shown_count, re_requested, raw_bytes, filtered_bytes, reduction_bps, plugin_name, output_format)| {
                ConversationEntry {
                    item_type,
                    item_id,
                    estimated_tokens,
                    commands_since_at_create,
                    first_shown,
                    last_shown,
                    shown_count,
                    re_requested,
                    raw_bytes,
                    filtered_bytes,
                    reduction_bps,
                    plugin_name,
                    output_format,
                }
            },
        ))
    }

    /// Record a conversation cache entry (insert or update).
    #[allow(clippy::too_many_arguments)]
    pub async fn record_conversation(
        &self,
        item_type: &str,
        item_id: &str,
        tokens: Option<i32>,
        commands_since: i64,
        raw_bytes: Option<i64>,
        filtered_bytes: Option<i64>,
        plugin_name: Option<String>,
        output_format: Option<String>,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        // Compute reduction percentage using rational arithmetic to avoid f64 cast.
        // Store as basis points (1/100 of a percent) for integer precision.
        let reduction_bps: Option<i64> = match (raw_bytes, filtered_bytes) {
            (Some(raw), Some(filtered)) if raw > 0 => {
                Some((raw.saturating_sub(filtered)).saturating_mul(10_000) / raw)
            }
            _ => None,
        };

        sqlx::query(
            "INSERT INTO conversation_cache (item_type, item_id, estimated_tokens, commands_since_at_create, first_shown, last_shown, raw_bytes, filtered_bytes, reduction_bps, plugin_name, output_format)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(item_type, item_id) DO UPDATE SET
                last_shown = excluded.last_shown,
                shown_count = shown_count + 1,
                raw_bytes = excluded.raw_bytes,
                filtered_bytes = excluded.filtered_bytes,
                reduction_bps = excluded.reduction_bps,
                plugin_name = excluded.plugin_name,
                output_format = excluded.output_format",
        )
        .bind(item_type)
        .bind(item_id)
        .bind(tokens)
        .bind(commands_since)
        .bind(now)
        .bind(raw_bytes)
        .bind(filtered_bytes)
        .bind(reduction_bps)
        .bind(&plugin_name)
        .bind(&output_format)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment the `re_requested` counter for a conversation entry.
    pub async fn increment_re_requested(&self, item_type: &str, item_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE conversation_cache SET re_requested = re_requested + 1
             WHERE item_type = ?1 AND item_id = ?2",
        )
        .bind(item_type)
        .bind(item_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clear all cache entries for a session.
    pub async fn clear_session(&self) -> Result<()> {
        sqlx::query("DELETE FROM conversation_cache").execute(&self.pool).await?;
        sqlx::query("DELETE FROM file_cache").execute(&self.pool).await?;
        Ok(())
    }

    /// Check if a cache key exists for the given session.
    pub async fn cache_has(&self, key: &str, session_id: &str) -> Result<bool> {
        let row = sqlx::query_scalar::<_, i32>(
            "SELECT 1 FROM sift_cache WHERE key = ?1 AND session_id = ?2",
        )
        .bind(key)
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    /// Record a cache key for the given session.
    pub async fn cache_set(&self, key: &str, session_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query(
            "INSERT INTO sift_cache (key, session_id, created_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key, session_id) DO NOTHING",
        )
        .bind(key)
        .bind(session_id)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Clear all cache entries for the given session.
    pub async fn cache_reset(&self, session_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sift_cache WHERE session_id = ?1")
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_from_env_no_session() {
        let session = Session::from_env();
        assert!(session.store.is_none());
        assert_eq!(session.cmd_count, 0);
    }

    #[tokio::test]
    async fn test_session_store_open_and_close() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();
        drop(store);
    }

    #[tokio::test]
    async fn test_file_cache_miss() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();
        let result = store.get_file_cache("/nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_file_cache_upsert_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        let entry = FileCacheEntry {
            path: "/tmp/test.txt".to_string(),
            hash: "abc123".to_string(),
            mtime: 1_000_000,
            size: 100,
            last_read: 2_000_000,
            read_count: 1,
        };
        store.upsert_file_cache(&entry).await.unwrap();

        let result = store.get_file_cache("/tmp/test.txt").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().hash, "abc123");
    }

    #[tokio::test]
    async fn test_conversation_cache() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store
            .record_conversation("file_content", "test:id", Some(100), 0, None, None, None, None)
            .await
            .unwrap();

        let result = store
            .get_conversation("file_content", "test:id")
            .await
            .unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_increment_re_requested() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store
            .record_conversation("file_content", "test:id", Some(100), 0, None, None, None, None)
            .await
            .unwrap();
        store
            .increment_re_requested("file_content", "test:id")
            .await
            .unwrap();

        let result = store
            .get_conversation("file_content", "test:id")
            .await
            .unwrap();
        assert_eq!(result.unwrap().re_requested, 1);
    }

    #[tokio::test]
    async fn test_clear_session() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store
            .record_conversation("file_content", "test:id", Some(100), 0, None, None, None, None)
            .await
            .unwrap();
        store.clear_session().await.unwrap();

        let result = store
            .get_conversation("file_content", "test:id")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_has_miss() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();
        let result = store.cache_has("path:hash", "session-a").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_cache_set_and_has() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store.cache_set("path:hash", "session-a").await.unwrap();

        // Same session: hit
        assert!(store.cache_has("path:hash", "session-a").await.unwrap());
        // Different session: miss
        assert!(!store.cache_has("path:hash", "session-b").await.unwrap());
        // Different key: miss
        assert!(!store.cache_has("other:hash", "session-a").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_reset() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store.cache_set("k1", "session-a").await.unwrap();
        store.cache_set("k2", "session-a").await.unwrap();
        store.cache_set("k1", "session-b").await.unwrap();

        store.cache_reset("session-a").await.unwrap();

        // Session A entries gone
        assert!(!store.cache_has("k1", "session-a").await.unwrap());
        assert!(!store.cache_has("k2", "session-a").await.unwrap());
        // Session B entries untouched
        assert!(store.cache_has("k1", "session-b").await.unwrap());
    }
}
