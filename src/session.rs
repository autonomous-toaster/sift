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
    /// Optional SQLite-backed session store.
    pub store: Option<SessionStore>,
}

impl Session {
    /// Create a new session from the environment.
    ///
    /// Session store is not opened. Call `open_store().await` to open it.
    pub fn from_env() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let env: HashMap<String, String> = std::env::vars().collect();

        Self {
            cwd,
            env,
            cmd_count: 0,
            store: None,
        }
    }

    /// Open the session store if `AI_SESSION` is set.
    pub async fn open_store(&mut self) {
        if let Ok(_session_id) = std::env::var("AI_SESSION") {
            let db_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".baish")
                .join("sessions.db");
            match SessionStore::open(&db_path).await {
                Ok(store) => {
                    self.store = Some(store);
                }
                Err(e) => {
                    eprintln!("baish: failed to open session store: {e}");
                }
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

/// A conversation cache entry.
#[derive(Debug, Clone)]
pub struct ConversationEntry {
    /// Type of item ('`file_content`', '`command_output`').
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
}

/// SQLite-backed session store.
pub struct SessionStore {
    pool: SqlitePool,
}

impl SessionStore {
    /// Open or create a session store at the given path.
    pub async fn open(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
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

        // Run migrations
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
                PRIMARY KEY (item_type, item_id)
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
        let row = sqlx::query_as::<_, (String, String, Option<i32>, i64, i64, i64, i32, i32)>(
            "SELECT item_type, item_id, estimated_tokens, commands_since_at_create,
                    first_shown, last_shown, shown_count, re_requested
             FROM conversation_cache
             WHERE item_type = ?1 AND item_id = ?2",
        )
        .bind(item_type)
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(item_type, item_id, estimated_tokens, commands_since_at_create, first_shown, last_shown, shown_count, re_requested)| {
                ConversationEntry {
                    item_type,
                    item_id,
                    estimated_tokens,
                    commands_since_at_create,
                    first_shown,
                    last_shown,
                    shown_count,
                    re_requested,
                }
            },
        ))
    }

    /// Record a conversation cache entry (insert or update).
    pub async fn record_conversation(
        &self,
        item_type: &str,
        item_id: &str,
        tokens: Option<i32>,
        commands_since: i64,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            "INSERT INTO conversation_cache (item_type, item_id, estimated_tokens, commands_since_at_create, first_shown, last_shown)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(item_type, item_id) DO UPDATE SET
                last_shown = excluded.last_shown,
                shown_count = shown_count + 1",
        )
        .bind(item_type)
        .bind(item_id)
        .bind(tokens)
        .bind(commands_since)
        .bind(now)
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
    async fn test_conversation_cache_miss() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        let result = store
            .get_conversation("file_content", "nonexistent")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_conversation_cache_record_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store
            .record_conversation("file_content", "test:id", Some(100), 0)
            .await
            .unwrap();

        let result = store
            .get_conversation("file_content", "test:id")
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().shown_count, 0);
    }

    #[tokio::test]
    async fn test_increment_re_requested() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::open(&db_path).await.unwrap();

        store
            .record_conversation("file_content", "test:id", Some(100), 0)
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
}
