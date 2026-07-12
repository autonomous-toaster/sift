//! Cat plugin — caches file reads, detects unchanged files.

use std::path::Path;

use anyhow::{Context, Result};
use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::plugin::{Plugin, PluginResult};
use crate::session::{FileCacheEntry, Session};

const STALENESS_THRESHOLD: u64 = 50;

/// Plugin for the `cat` command.
pub struct CatPlugin;

#[async_trait]
impl Plugin for CatPlugin {
    fn name(&self) -> &'static str {
        "cat"
    }

    async fn execute(
        &self,
        session: &mut Session,
        args: &[String],
        stdin: Option<&[u8]>,
    ) -> Result<PluginResult> {
        if stdin.is_some() {
            return Ok(PluginResult::Passthrough);
        }

        if args.iter().any(|a| a.starts_with('-')) {
            return Ok(PluginResult::Passthrough);
        }

        if args.is_empty() || args.len() > 1 {
            return Ok(PluginResult::Passthrough);
        }

        let path_str = &args[0];
        let path = Path::new(path_str);

        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            session.cwd.join(path)
        };

        let content = std::fs::read(&full_path)
            .with_context(|| format!("cat: {path_str}: No such file or directory"))?;

        let hash = hex::encode(Sha256::digest(&content));

        if let Some(result) = check_cache(session, &full_path, &hash, &content, path_str).await? {
            return Ok(result);
        }

        update_cache(session, &full_path, &hash, &content).await?;

        Ok(PluginResult::Handled {
            output: content,
            exit_code: 0,
        })
    }
}

async fn check_cache(
    session: &Session,
    full_path: &Path,
    hash: &str,
    content: &[u8],
    path_str: &str,
) -> Result<Option<PluginResult>> {
    let Some(ref store) = session.store else {
        return Ok(None);
    };

    let cache_key = format!("{}:{}", full_path.display(), hash);
    let entry = store.get_conversation("file_content", &cache_key).await?;

    let entry = match entry {
        Some(e) if e.shown_count > 0 => e,
        _ => return Ok(None),
    };

    let cmds_since = session
        .cmd_count
        .saturating_sub(entry.commands_since_at_create.cast_unsigned());

    if cmds_since >= STALENESS_THRESHOLD {
        return Ok(None);
    }

    if entry.re_requested > 0 {
        store
            .record_conversation(
                "file_content",
                &cache_key,
                Some(estimate_tokens(content)),
                session.cmd_count.cast_signed(),
            )
            .await?;
        return Ok(Some(PluginResult::Handled {
            output: content.to_vec(),
            exit_code: 0,
        }));
    }

    Ok(Some(PluginResult::Unchanged {
        fingerprint: cache_key,
        message: format!(
            "[baish] {path_str} unchanged since last read ({cmds_since} commands ago)"
        ),
    }))
}

async fn update_cache(
    session: &Session,
    full_path: &Path,
    hash: &str,
    content: &[u8],
) -> Result<()> {
    let Some(ref store) = session.store else {
        return Ok(());
    };

    let cache_key = format!("{}:{}", full_path.display(), hash);
    let metadata = std::fs::metadata(full_path)?;
    let file_entry = FileCacheEntry {
        path: full_path.display().to_string(),
        hash: hash.to_string(),
        mtime: metadata
            .modified()
            .map_or(0, |t| {
                i64::try_from(
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                )
                .unwrap_or(0)
            }),
        size: metadata.len().cast_signed(),
        last_read: chrono::Utc::now().timestamp_millis(),
        read_count: 1,
    };
    store.upsert_file_cache(&file_entry).await?;

    store
        .record_conversation(
            "file_content",
            &cache_key,
            Some(estimate_tokens(content)),
            session.cmd_count.cast_signed(),
        )
        .await?;

    Ok(())
}

fn estimate_tokens(content: &[u8]) -> i32 {
    i32::try_from(content.len().saturating_div(4)).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cat_single_file() {
        let mut session = Session::from_env();
        let plugin = CatPlugin;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();

        let result = plugin
            .execute(&mut session, &[file_path.display().to_string()], None)
            .await
            .unwrap();

        match result {
            PluginResult::Handled { output, .. } => {
                assert_eq!(output, b"hello world");
            }
            _ => panic!("expected Handled"),
        }
    }

    #[tokio::test]
    async fn test_cat_with_flags_passthrough() {
        let mut session = Session::from_env();
        let plugin = CatPlugin;

        let result = plugin
            .execute(&mut session, &["-n".to_string(), "file.txt".to_string()], None)
            .await;
        assert!(matches!(result.unwrap(), PluginResult::Passthrough));
    }

    #[tokio::test]
    async fn test_cat_nonexistent_file() {
        let mut session = Session::from_env();
        let plugin = CatPlugin;

        let result = plugin
            .execute(&mut session, &["/nonexistent/file.txt".to_string()], None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cat_multiple_files_passthrough() {
        let mut session = Session::from_env();
        let plugin = CatPlugin;

        let result = plugin
            .execute(
                &mut session,
                &["file1.txt".to_string(), "file2.txt".to_string()],
                None,
            )
            .await;
        assert!(matches!(result.unwrap(), PluginResult::Passthrough));
    }

    #[tokio::test]
    async fn test_cat_with_stdin_passthrough() {
        let mut session = Session::from_env();
        let plugin = CatPlugin;

        let result = plugin
            .execute(&mut session, &["file.txt".to_string()], Some(b"input"))
            .await;
        assert!(matches!(result.unwrap(), PluginResult::Passthrough));
    }

    #[tokio::test]
    async fn test_cat_repeated_read() {
        let mut session = Session::from_env();
        let plugin = CatPlugin;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();

        let result1 = plugin
            .execute(&mut session, &[file_path.display().to_string()], None)
            .await
            .unwrap();
        assert!(matches!(result1, PluginResult::Handled { .. }));

        let result2 = plugin
            .execute(&mut session, &[file_path.display().to_string()], None)
            .await
            .unwrap();
        assert!(matches!(result2, PluginResult::Handled { .. }));
    }
}
