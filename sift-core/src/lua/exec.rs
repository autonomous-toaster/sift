//! Command execution and cache cleanup utilities.
//!
//! Provides `exec_command()` for running bash commands with streaming output,
//! `save_output()` for persisting raw output, and `cleanup_cache()` for
//! pruning expired cache entries.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json;

/// Find the real bash binary, excluding our own path.
pub(crate) fn find_real_bash() -> PathBuf {
    let self_path = std::env::current_exe().ok();
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join("bash");
        if candidate.is_file() {
            if let Ok(canonical) = candidate.canonicalize() {
                if self_path.as_ref().is_some_and(|s| s == &canonical) {
                    continue;
                }
                return canonical;
            }
        }
    }
    for fallback in &["/bin/bash", "/usr/bin/bash", "/usr/local/bin/bash"] {
        let p = PathBuf::from(fallback);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("/bin/bash")
}

/// Transform function for streaming output: receives a chunk, returns (possibly modified) chunk.
pub(crate) type TransformFn = Box<dyn Fn(&str) -> String + Send>;

/// Execute a command via `std::process::Command` with pipes and return `(stdout, stderr, exit_code)`.
///
/// Streams stdout/stderr to the real stdout/stderr in real-time while collecting for the return value.
/// If `transform` is provided, each stdout chunk is passed through it before writing and collecting.
pub(crate) fn exec_command(
    cmd: &str,
    _session_id: &str,
    _cmd_count: u64,
    transform: Option<TransformFn>,
    silent: bool,
    _merge_stderr: bool,
) -> Result<(String, String, i32), mlua::Error> {
    let bash_path = find_real_bash();

    // Fast path: no transform, use output() to avoid thread overhead
    if transform.is_none() {
        let output = std::process::Command::new(&bash_path)
            .arg("-c")
            .arg(cmd)
            .env("PAGER", "cat")
            .env("TERM", "dumb")
            .env("EDITOR", "true")
            .env("GIT_EDITOR", "true")
            .env("GIT_PAGER", "cat")
            .output()
            .map_err(|e| mlua::Error::external(format!("spawn: {e}")))?;
        let stdout = String::from_utf8(output.stdout)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());
        let stderr = String::from_utf8(output.stderr)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());
        let exit_code = output.status.code().unwrap_or(1);
        if !silent {
            let _ = std::io::stdout().write(stdout.as_bytes());
            let _ = std::io::stderr().write(stderr.as_bytes());
        }
        return Ok((stdout, stderr, exit_code));
    }

    // Slow path: transform provided, use threaded streaming
    let mut child = std::process::Command::new(&bash_path)
        .arg("-c")
        .arg(cmd)
        .env("PAGER", "cat")
        .env("TERM", "dumb")
        .env("EDITOR", "true")
        .env("GIT_EDITOR", "true")
        .env("GIT_PAGER", "cat")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| mlua::Error::external(format!("spawn: {e}")))?;

    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| mlua::Error::external("no stdout pipe".to_string()))?;
    let stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| mlua::Error::external("no stderr pipe".to_string()))?;

    let stdout_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let stderr_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let stdout_buf_clone = Arc::clone(&stdout_buf);
    let stdout_handle = std::thread::spawn(move || {
        let mut reader = BufReader::with_capacity(65536, stdout_pipe);
        let mut collected = String::with_capacity(4096);
        loop {
            let buf = match reader.fill_buf() {
                Ok(buf) => buf,
                Err(e) => {
                    eprintln!("sift: stdout read error: {e}");
                    break;
                }
            };
            if buf.is_empty() {
                break;
            }
            let n = buf.len();
            if let Some(ref t) = transform {
                let s = String::from_utf8_lossy(buf);
                let output = t(&s);
                if !silent {
                    let _ = std::io::stdout().write(output.as_bytes());
                }
                collected.push_str(&output);
            } else {
                if !silent {
                    let _ = std::io::stdout().write(buf);
                }
                collected.push_str(&String::from_utf8_lossy(buf));
            }
            reader.consume(n);
        }
        if let Ok(mut guard) = stdout_buf_clone.lock() {
            *guard = collected;
        }
    });

    let stderr_buf_clone = Arc::clone(&stderr_buf);
    let stderr_handle = std::thread::spawn(move || {
        let mut reader = BufReader::with_capacity(65536, stderr_pipe);
        let mut collected = String::with_capacity(4096);
        loop {
            let buf = match reader.fill_buf() {
                Ok(buf) => buf,
                Err(e) => {
                    eprintln!("sift: stderr read error: {e}");
                    break;
                }
            };
            if buf.is_empty() {
                break;
            }
            let n = buf.len();
            let s = String::from_utf8_lossy(buf);
            if !silent {
                let _ = std::io::stderr().write(s.as_bytes());
            }
            collected.push_str(&s);
            reader.consume(n);
        }
        if let Ok(mut guard) = stderr_buf_clone.lock() {
            *guard = collected;
        }
    });

    let status = child
        .wait()
        .map_err(|e| mlua::Error::external(format!("wait: {e}")))?;
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    let stdout = stdout_buf.lock().map(|g| g.clone()).unwrap_or_default();
    let stderr = stderr_buf.lock().map(|g| g.clone()).unwrap_or_default();
    let exit_code = status.code().unwrap_or(1);

    Ok((stdout, stderr, exit_code))
}

/// Save raw output to a temp file and return the path.
pub(crate) fn save_output(cmd: &str, session_id: &str, cmd_count: u64, output: &str) -> String {
    let slug: String = cmd
        .chars()
        .take(40)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(session_id);
    let _ = std::fs::create_dir_all(&tmp_dir);
    let log_path = tmp_dir.join(format!("{ts}_{cmd_count}_{slug}.log"));
    let _ = std::fs::write(&log_path, output);
    log_path.display().to_string()
}

/// Clean up expired cache entries and orphan objects for a session.
/// Runs at startup to prevent unbounded cache growth.
pub fn cleanup_cache(session_id: &str, max_age_ms: u64) {
    let base = std::path::PathBuf::from("/tmp/sift").join(session_id);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let cache_dir = base.join("cache");
    let mut active_hashes: Vec<String> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy().to_string();
            if let Ok(meta_str) = std::fs::read_to_string(entry.path()) {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                    let created = meta["created_at"].as_u64().unwrap_or(0);
                    if now.saturating_sub(u128::from(created)) > u128::from(max_age_ms) {
                        let _ = std::fs::remove_file(entry.path());
                        let obj_path = base.join("objects").join(format!("sha256-{fname_str}.txt"));
                        let _ = std::fs::remove_file(&obj_path);
                        continue;
                    }
                }
            }
            active_hashes.push(fname_str);
        }
    }

    let objects_dir = base.join("objects");
    if let Ok(entries) = std::fs::read_dir(&objects_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if let Some(hash) = fname
                .strip_prefix("sha256-")
                .and_then(|s| s.strip_suffix(".txt"))
            {
                if !active_hashes.contains(&hash.to_string()) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
}
