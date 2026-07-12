//! Shell builtins: cd, export, unset, source, exit.

use std::path::Path;

use anyhow::{Context, Result};

use crate::plugin::PluginRegistry;
use crate::session::Session;

/// Check if a command is a builtin.
pub fn is_builtin(cmd: &str) -> bool {
    matches!(cmd, "cd" | "export" | "unset" | "source" | "." | "exit")
}

/// Execute a builtin command.
///
/// Returns `Some(output)` if the command was handled, `None` if not a builtin.
pub async fn execute_builtin(
    session: &mut Session,
    registry: &PluginRegistry,
    cmd: &str,
    args: &[String],
) -> Result<Option<Vec<u8>>> {
    match cmd {
        "cd" => Ok(Some(execute_cd(session, args)?)),
        "export" => Ok(Some(execute_export(session, args))),
        "unset" => Ok(Some(execute_unset(session, args))),
        "source" | "." => Box::pin(execute_source(session, registry, args)).await,
        "exit" => {
            let code = args.first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            std::process::exit(code);
        }
        _ => Ok(None),
    }
}

fn execute_cd(session: &mut Session, args: &[String]) -> Result<Vec<u8>> {
    let target = if args.is_empty() {
        session
            .env
            .get("HOME")
            .cloned()
            .unwrap_or_else(|| "/".to_string())
    } else if args[0] == "-" {
        match session.env.get("OLDPWD") {
            Some(dir) => {
                let output = format!("{dir}\n").into_bytes();
                let old = session.cwd.clone();
                let new = Path::new(dir).to_path_buf();
                session.cwd = new;
                session.env.insert("OLDPWD".to_string(), old.display().to_string());
                return Ok(output);
            }
            None => {
                return Err(anyhow::anyhow!("OLDPWD not set"));
            }
        }
    } else {
        args[0].clone()
    };

    let old = session.cwd.clone();
    let new = if let Some(stripped) = target.strip_prefix('~') {
        let home = session.env.get("HOME").cloned().unwrap_or_else(|| "/".to_string());
        Path::new(&home).join(stripped.strip_prefix('/').unwrap_or(stripped))
    } else if target.starts_with('/') {
        Path::new(&target).to_path_buf()
    } else {
        session.cwd.join(&target)
    };

    let canonical = new
        .canonicalize()
        .with_context(|| format!("cd: {target}: No such file or directory"))?;

    session.cwd = canonical;
    session
        .env
        .insert("OLDPWD".to_string(), old.display().to_string());

    Ok(Vec::new())
}

fn execute_export(session: &mut Session, args: &[String]) -> Vec<u8> {
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            session.env.insert(key.to_string(), value.to_string());
        } else if !arg.is_empty() {
            session.env.entry(arg.clone()).or_default();
        }
    }
    Vec::new()
}

fn execute_unset(session: &mut Session, args: &[String]) -> Vec<u8> {
    for arg in args {
        session.env.remove(arg);
    }
    Vec::new()
}

async fn execute_source(
    session: &mut Session,
    registry: &PluginRegistry,
    args: &[String],
) -> Result<Option<Vec<u8>>> {
    let path = args.first().context("source: missing filename")?;
    let resolved = if path.starts_with('/') {
        Path::new(path).to_path_buf()
    } else {
        session.cwd.join(path)
    };

    let content = std::fs::read_to_string(&resolved)
        .with_context(|| format!("source: {path}: No such file or directory"))?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Err(e) = crate::dispatcher::dispatch(session, registry, line).await {
            eprintln!("baish: source error in {path}: {e}");
        }
    }

    Ok(Some(Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginRegistry;

    fn test_session() -> Session {
        let mut session = Session::from_env();
        session.cwd = std::env::temp_dir().canonicalize().unwrap();
        session.env.insert(
            "HOME".to_string(),
            dirs::home_dir().unwrap().display().to_string(),
        );
        session
    }

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("cd"));
        assert!(is_builtin("export"));
        assert!(is_builtin("unset"));
        assert!(is_builtin("source"));
        assert!(is_builtin("."));
        assert!(is_builtin("exit"));
        assert!(!is_builtin("cat"));
        assert!(!is_builtin("git"));
    }

    #[test]
    fn test_cd_absolute() {
        let mut session = test_session();
        let tmp = std::env::temp_dir().canonicalize().unwrap().display().to_string();
        execute_cd(&mut session, std::slice::from_ref(&tmp)).unwrap();
        assert_eq!(session.cwd.display().to_string(), tmp);
    }

    #[test]
    fn test_cd_home() {
        let mut session = test_session();
        let home = session.env.get("HOME").cloned().unwrap();
        execute_cd(&mut session, &[]).unwrap();
        assert_eq!(session.cwd.display().to_string(), home);
    }

    #[test]
    fn test_cd_back() {
        let mut session = test_session();
        let original = session.cwd.canonicalize().unwrap().display().to_string();
        let tmp = std::env::temp_dir().canonicalize().unwrap().display().to_string();

        execute_cd(&mut session, std::slice::from_ref(&tmp)).unwrap();
        assert_eq!(session.cwd.display().to_string(), tmp);

        execute_cd(&mut session, &["-".to_string()]).unwrap();
        assert_eq!(session.cwd.display().to_string(), original);
    }

    #[test]
    fn test_export() {
        let mut session = test_session();
        execute_export(&mut session, &["FOO=bar".to_string()]);
        assert_eq!(session.env.get("FOO").unwrap(), "bar");
    }

    #[test]
    fn test_unset() {
        let mut session = test_session();
        session.env.insert("FOO".to_string(), "bar".to_string());
        execute_unset(&mut session, &["FOO".to_string()]);
        assert!(!session.env.contains_key("FOO"));
    }

    #[tokio::test]
    async fn test_source_nonexistent() {
        let mut session = test_session();
        let registry = PluginRegistry::new();
        let result = execute_source(
            &mut session,
            &registry,
            &["/nonexistent/file.sh".to_string()],
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_builtin_cd() {
        let mut session = test_session();
        let registry = PluginRegistry::new();
        let tmp = std::env::temp_dir().canonicalize().unwrap().display().to_string();
        let result = execute_builtin(&mut session, &registry, "cd", std::slice::from_ref(&tmp)).await;
        assert!(result.is_ok());
        assert_eq!(session.cwd.display().to_string(), tmp);
    }

    #[tokio::test]
    async fn test_execute_builtin_export() {
        let mut session = test_session();
        let registry = PluginRegistry::new();
        let result = execute_builtin(&mut session, &registry, "export", &["FOO=bar".to_string()]).await;
        assert!(result.is_ok());
        assert_eq!(session.env.get("FOO").unwrap(), "bar");
    }

    #[tokio::test]
    async fn test_execute_builtin_unset() {
        let mut session = test_session();
        let registry = PluginRegistry::new();
        session.env.insert("FOO".to_string(), "bar".to_string());
        let result = execute_builtin(&mut session, &registry, "unset", &["FOO".to_string()]).await;
        assert!(result.is_ok());
        assert!(!session.env.contains_key("FOO"));
    }

    #[tokio::test]
    async fn test_execute_builtin_unknown() {
        let mut session = test_session();
        let registry = PluginRegistry::new();
        let result = execute_builtin(&mut session, &registry, "cat", &[]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
