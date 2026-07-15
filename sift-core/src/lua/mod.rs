//! Lua runtime and `sift.*` API for plugins.
//!
//! Provides the mlua-based Lua VM, registers all `sift.*` functions,
//! and handles plugin loading and dispatch.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use mlua::Lua;

use crate::session::SessionStore;

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
) -> Result<(String, String, i32), mlua::Error> {
    let bash_path = find_real_bash();
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

    let stdout_pipe = child.stdout.take()
        .ok_or_else(|| mlua::Error::external("no stdout pipe".to_string()))?;
    let stderr_pipe = child.stderr.take()
        .ok_or_else(|| mlua::Error::external("no stderr pipe".to_string()))?;

    let stdout_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let stderr_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let stdout_buf_clone = Arc::clone(&stdout_buf);
    let stdout_handle = std::thread::spawn(move || {
        let mut reader = stdout_pipe;
        let mut chunk = [0u8; 4096];
        let mut collected = String::new();
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&chunk[..n]).to_string();
                    let output = transform.as_ref().map_or_else(|| s.clone(), |t| t(&s));
                    print!("{output}");
                    let _ = std::io::stdout().flush();
                    collected.push_str(&output);
                }
                Err(e) => {
                    eprintln!("sift: stdout read error: {e}");
                    break;
                }
            }
        }
        if let Ok(mut guard) = stdout_buf_clone.lock() {
            *guard = collected;
        }
    });

    let stderr_buf_clone = Arc::clone(&stderr_buf);
    let stderr_handle = std::thread::spawn(move || {
        let mut reader = stderr_pipe;
        let mut chunk = [0u8; 4096];
        let mut collected = String::new();
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&chunk[..n]).to_string();
                    eprint!("{s}");
                    let _ = std::io::stderr().flush();
                    collected.push_str(&s);
                }
                Err(e) => {
                    eprintln!("sift: stderr read error: {e}");
                    break;
                }
            }
        }
        if let Ok(mut guard) = stderr_buf_clone.lock() {
            *guard = collected;
        }
    });

    let status = child.wait().map_err(|e| mlua::Error::external(format!("wait: {e}")))?;
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    let stdout = stdout_buf.lock().map(|g| g.clone()).unwrap_or_default();
    let stderr = stderr_buf.lock().map(|g| g.clone()).unwrap_or_default();
    let exit_code = status.code().unwrap_or(1);

    Ok((stdout, stderr, exit_code))
}

/// Save raw output to a temp file and return the path.
pub(crate) fn save_output(cmd: &str, session_id: &str, cmd_count: u64, output: &str) -> String {
    let slug: String = cmd.chars()
        .take(40)
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
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

    // Scan cache markers, delete expired
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

    // Scan objects, delete orphans
    let objects_dir = base.join("objects");
    if let Ok(entries) = std::fs::read_dir(&objects_dir) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if let Some(hash) = fname.strip_prefix("sha256-").and_then(|s| s.strip_suffix(".txt")) {
                if !active_hashes.contains(&hash.to_string()) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
}

/// The sift Lua runtime, holding the VM and registered API.
pub struct SiftLua {
    /// The Lua VM.
    lua: Lua,
    /// Registered plugins: `(pattern, priority, plugin_table)`.
    plugins: Vec<PluginEntry>,
    /// Session store for cache operations.
    store: Option<Arc<SessionStore>>,
    /// Current session context.
    ctx: SiftContext,
    /// Nudge messages accumulated during plugin execution.
    nudges: Arc<Mutex<Vec<String>>>,
}

/// Context passed to plugin execution.
#[derive(Clone, Debug)]
pub struct SiftContext {
    /// Current working directory.
    pub cwd: PathBuf,
    /// Command counter.
    pub cmd_count: u64,
    /// Environment variables.
    pub env: HashMap<String, String>,
    /// Session identifier.
    pub session_id: Option<String>,
    /// Raw output bytes (set by plugin or computed).
    pub raw_bytes: u64,
    /// Filtered output bytes (computed from returned output).
    pub filtered_bytes: u64,
}

/// A registered plugin entry.
pub(crate) struct PluginEntry {
    /// Command patterns for matching (e.g., `["cat"]`, `["docker", "podman"]`).
    patterns: Vec<String>,
    /// Priority: higher wins on tie.
    priority: i32,
    /// The Lua plugin table reference.
    table: mlua::RegistryKey,
}

impl SiftLua {
    /// Create a new Lua runtime and register all `sift.*` API functions.
    pub fn new(store: Option<Arc<SessionStore>>, ctx: SiftContext) -> Result<Self> {
        let lua = Lua::new();

        let runtime = Self {
            lua,
            plugins: Vec::new(),
            store,
            ctx,
            nudges: Arc::new(Mutex::new(Vec::new())),
        };

        runtime.register_sift_table()?;
        Ok(runtime)
    }
}

pub(crate) mod api;

#[cfg(test)]
mod tests {
    use super::{exec_command, find_real_bash, save_output, SiftContext, SiftLua};
    use mlua::{Lua, Table};
    use serde_json;
    use std::collections::HashMap;

    fn test_context() -> SiftContext {
        SiftContext {
            cwd: std::env::current_dir().unwrap(),
            cmd_count: 0,
            env: HashMap::new(),
            session_id: None,
            raw_bytes: 0,
            filtered_bytes: 0,
        }
    }

    fn test_ctx(lua: &Lua) -> Table {
        let ctx = lua.create_table().unwrap();
        ctx.set("session_id", "test").unwrap();
        ctx.set("cmd_count", 0).unwrap();
        ctx.set("cwd", "/tmp").unwrap();
        ctx.set("command", "test").unwrap();
        ctx
    }

    #[test]
    fn test_sift_lua_creation() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        assert!(sift.get::<mlua::Function>("exec").is_ok());
        assert!(sift.get::<mlua::Table>("log").is_ok());
        assert!(sift.get::<mlua::Table>("hash").is_ok());
        assert!(sift.get::<mlua::Table>("json").is_ok());
        assert!(sift.get::<mlua::Table>("meta").is_ok());
    }

    #[test]
    fn test_sift_hash_sha256() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let hash: Table = sift.get("hash").unwrap();
        let sha256: mlua::Function = hash.get("sha256").unwrap();
        let result: String = sha256.call((test_ctx(&lua.lua), "hello")).unwrap();
        assert_eq!(result, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_sift_token_count() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let token_count: mlua::Function = sift.get("token_count").unwrap();
        let result: isize = token_count.call((test_ctx(&lua.lua), "hello world")).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn test_plugin_load_and_dispatch() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        let plugin_code = r#"
            return {
                name = "test-cmd",
                priority = 0,
                pattern = "test-cmd",
                execute = function(ctx, args, stdin)
                    return { status = "handled", output = "test: " .. (args[1] or "none"), exit_code = 0 }
                end
            }
        "#;
        lua.load_plugin_from_str("test", plugin_code).unwrap();
        let (output, exit_code, plugin) = lua.dispatch("test-cmd", &["arg1".to_string()], None).unwrap();
        assert_eq!(output, "test: arg1");
        assert_eq!(exit_code, 0);
        assert_eq!(plugin, "test-cmd");
    }

    #[test]
    fn test_plugin_dispatch_fallback() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        let plugin_code = r#"
            return {
                name = "__default__",
                priority = -1000,
                pattern = "__default__",
                execute = function(ctx, args, stdin)
                    return { status = "handled", output = "fallback", exit_code = 0 }
                end
            }
        "#;
        lua.load_plugin_from_str("default", plugin_code).unwrap();
        let (output, _exit_code, _plugin) = lua.dispatch("unknown-cmd", &[], None).unwrap();
        assert_eq!(output, "fallback");
    }

    #[test]
    fn test_plugin_priority_ordering() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        let low = r#"return { name = "test", priority = -100, pattern = "test", execute = function() return { status = "handled", output = "low", exit_code = 0 } end }"#;
        let high = r#"return { name = "test", priority = 100, pattern = "test", execute = function() return { status = "handled", output = "high", exit_code = 0 } end }"#;
        lua.load_plugin_from_str("low", low).unwrap();
        lua.load_plugin_from_str("high", high).unwrap();
        let (output, _exit_code, _plugin) = lua.dispatch("test", &[], None).unwrap();
        assert_eq!(output, "high");
    }

    #[test]
    fn test_sift_fs_read() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let fs: Table = sift.get("fs").unwrap();
        let fs_read: mlua::Function = fs.get("read").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello world").unwrap();
        let content: String = fs_read.call((test_ctx(&lua.lua), path.display().to_string(), mlua::Value::Nil)).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_sift_json_encode() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let encode: mlua::Function = json.get("encode").unwrap();
        let tbl = lua.lua.create_table().unwrap();
        tbl.set("name", "test").unwrap();
        let encoded: String = encode.call((test_ctx(&lua.lua), tbl)).unwrap();
        assert!(encoded.contains("name"));
    }

    #[test]
    fn test_find_real_bash_exists() {
        let bash = find_real_bash();
        assert!(bash.exists(), "real bash should exist at {bash:?}");
    }

    #[test]
    fn test_sift_fs_write_and_read() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let fs: Table = sift.get("fs").unwrap();
        let fs_write: mlua::Function = fs.get("write").unwrap();
        let fs_read: mlua::Function = fs.get("read").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt").display().to_string();
        fs_write.call::<()>((test_ctx(&lua.lua), path.clone(), "hello world")).unwrap();
        let content: String = fs_read.call((test_ctx(&lua.lua), path, mlua::Value::Nil)).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_sift_fs_stat() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let fs: Table = sift.get("fs").unwrap();
        let fs_stat: mlua::Function = fs.get("stat").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();
        let result: Table = fs_stat.call((test_ctx(&lua.lua), path.display().to_string())).unwrap();
        let is_file: bool = result.get("is_file").unwrap();
        assert!(is_file);
    }

    #[test]
    fn test_sift_fs_exists() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let fs: Table = sift.get("fs").unwrap();
        let fs_exists: mlua::Function = fs.get("exists").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();
        assert!(fs_exists.call::<bool>((test_ctx(&lua.lua), path.display().to_string())).unwrap());
        assert!(!fs_exists.call::<bool>((test_ctx(&lua.lua), dir.path().join("nonexistent").display().to_string())).unwrap());
    }

    #[test]
    fn test_sift_json_shortest_raw_wins_small() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let shortest: mlua::Function = json.get("shortest").unwrap();

        let formats = lua.lua.create_table().unwrap();
        let json_opts = lua.lua.create_table().unwrap();
        json_opts.set("max_string_len", 80).unwrap();
        formats.set("json", json_opts).unwrap();
        formats.set("toon", true).unwrap();

        // Small JSON — raw should win (nudge overhead exceeds savings)
        let small = r#"{"name":"test","value":42}"#;
        let result: String = shortest.call((test_ctx(&lua.lua), small, formats)).unwrap();
        assert_eq!(result, small, "raw should win for small JSON");
    }

    #[test]
    fn test_sift_json_shortest_non_json() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let shortest: mlua::Function = json.get("shortest").unwrap();

        let formats = lua.lua.create_table().unwrap();
        formats.set("toon", true).unwrap();

        // Non-JSON — return raw unchanged
        let result: String = shortest.call((test_ctx(&lua.lua), "not json", formats)).unwrap();
        assert_eq!(result, "not json");
    }

    #[test]
    fn test_sift_json_shortest_empty() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let shortest: mlua::Function = json.get("shortest").unwrap();

        let formats = lua.lua.create_table().unwrap();
        formats.set("toon", true).unwrap();

        // Empty JSON object
        let result: String = shortest.call((test_ctx(&lua.lua), "{}", formats)).unwrap();
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_sift_json_shortest_tiny_json() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let shortest: mlua::Function = json.get("shortest").unwrap();

        let formats = lua.lua.create_table().unwrap();
        formats.set("toon", true).unwrap();

        // Tiny JSON — raw should win
        let result: String = shortest.call((test_ctx(&lua.lua), "42", formats)).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_sift_json_shortest_large_json_toon_wins() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let shortest: mlua::Function = json.get("shortest").unwrap();

        let formats = lua.lua.create_table().unwrap();
        formats.set("toon", true).unwrap();

        // Large JSON with many repeated fields — TOON should win
        let mut items = Vec::new();
        for i in 0..100 {
            items.push(serde_json::json!({
                "name": format!("item-{}", i),
                "value": i,
                "description": "a long string that takes up many tokens in json because of quotes and commas"
            }));
        }
        let large = serde_json::json!({"items": items});
        let large_str = serde_json::to_string(&large).unwrap();
        assert!(large_str.len() > 2000, "large JSON should be >2000 chars");

        let result: String = shortest.call((test_ctx(&lua.lua), large_str, formats)).unwrap();
        // For large JSON, TOON should win (more compact than raw + nudge overhead)
        assert!(!result.is_empty(), "shortest should return non-empty output");
        // TOON output typically contains colons and indentation
        let is_toon = result.contains(':') && result.contains("  ");
        assert!(is_toon, "large JSON should produce TOON output, got: {}", &result[..200.min(result.len())]);
    }

    #[test]
    fn test_sift_toon_encode() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let toon: Table = sift.get("toon").unwrap();
        let encode: mlua::Function = toon.get("encode").unwrap();
        let tbl = lua.lua.create_table().unwrap();
        tbl.set("name", "test").unwrap();
        let encoded: String = encode.call((test_ctx(&lua.lua), tbl)).unwrap();
        assert!(encoded.contains("name"));
    }

    #[test]
    fn test_sift_env() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let env: Table = sift.get("env").unwrap();
        let env_set: mlua::Function = env.get("set").unwrap();
        let env_get: mlua::Function = env.get("get").unwrap();
        env_set.call::<()>((test_ctx(&lua.lua), "SIFT_TEST", "val")).unwrap();
        let result: Option<String> = env_get.call((test_ctx(&lua.lua), "SIFT_TEST")).unwrap();
        assert_eq!(result, Some("val".to_string()));
    }

    #[test]
    fn test_sift_meta() {
        let ctx = SiftContext {
            cwd: std::env::current_dir().unwrap(),
            cmd_count: 42,
            env: HashMap::new(),
            session_id: Some("test-session".to_string()),
            raw_bytes: 100,
            filtered_bytes: 50,
        };
        let lua = SiftLua::new(None, ctx).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let meta: Table = sift.get("meta").unwrap();
        let session_id: String = meta.get("session_id").unwrap();
        assert_eq!(session_id, "test-session");
        let cmd_count: i64 = meta.get("cmd_count").unwrap();
        assert_eq!(cmd_count, 42);
    }

    #[test]
    fn test_exec_command() {
        let (stdout, stderr, code) = exec_command("echo hello", "test", 0, None).unwrap();
        assert!(stdout.contains("hello"), "stdout should contain hello, got: {stdout}");
        assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
        assert_eq!(code, 0);
    }

    #[test]
    fn test_exec_command_with_stderr() {
        let (stdout, stderr, code) = exec_command("echo out && echo err >&2", "test", 0, None).unwrap();
        assert!(stdout.contains("out"), "stdout should contain out, got: {stdout}");
        assert!(stderr.contains("err"), "stderr should contain err, got: {stderr}");
        assert_eq!(code, 0);
    }

    #[test]
    fn test_exec_command_exit_code() {
        let (_stdout, _stderr, code) = exec_command("exit 42", "test", 0, None).unwrap();
        assert_eq!(code, 42, "exit code should be 42, got {code}");
    }

    #[test]
    fn test_save_output() {
        let session_id = "test-save";
        save_output("echo test", session_id, 1, "test content");
        let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(session_id);
        let has_files = std::fs::read_dir(&tmp_dir).is_ok();
        assert!(has_files, "should have saved output files");
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_dispatch_full_simple() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        let plugin_code = r#"
            return {
                name = "test-cmd",
                priority = 0,
                pattern = "test-cmd",
                execute = function(ctx, args, stdin)
                    return { status = "handled", output = "ok", exit_code = 0 }
                end
            }
        "#;
        lua.load_plugin_from_str("test", plugin_code).unwrap();
        let (output, exit_code, _plugin) = lua.dispatch_full("test-cmd arg1", None).unwrap();
        assert_eq!(output, "ok");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_dispatch_full_empty() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let (output, exit_code, _plugin) = lua.dispatch_full("", None).unwrap();
        assert_eq!(output, "");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_split_pipeline_simple() {
        let segments = super::api::split_pipeline("echo abc | cat");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], "echo abc");
        assert_eq!(segments[1], "cat");
    }

    #[test]
    fn test_split_pipeline_logical_or() {
        let segments = super::api::split_pipeline("false || echo ok");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "false || echo ok");
    }

    #[test]
    fn test_split_pipeline_no_pipe() {
        let segments = super::api::split_pipeline("cat foo.rs");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "cat foo.rs");
    }

    #[test]
    fn test_dispatch_full_cd_prefix() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        let plugin_code = r#"
            return {
                name = "test-cmd",
                priority = 0,
                pattern = "test-cmd",
                execute = function(ctx, args, stdin)
                    return { status = "handled", output = "cd-dispatched", exit_code = 0 }
                end
            }
        "#;
        lua.load_plugin_from_str("test", plugin_code).unwrap();
        // cd /tmp && test-cmd should dispatch test-cmd
        let (output, exit_code, _plugin) = lua.dispatch_full("cd /tmp && test-cmd", None).unwrap();
        assert_eq!(output, "cd-dispatched");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_dispatch_full_pipeline_fallback() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        // Load a default plugin for fallback
        let default_code = r#"
            return {
                name = "__default__",
                priority = -1000,
                pattern = "__default__",
                execute = function(ctx, args, stdin)
                    return { status = "handled", output = "fallback", exit_code = 0 }
                end
            }
        "#;
        lua.load_plugin_from_str("default", default_code).unwrap();
        // Pipeline with no matching plugin should fall through to default handler
        let (output, exit_code, _plugin) = lua.dispatch_full("echo hello | grep hello", None).unwrap();
        assert_eq!(output, "fallback");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_dispatch_full_popd() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let (output, exit_code, _plugin) = lua.dispatch_full("popd", None).unwrap();
        assert_eq!(output, "");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_dispatch_unchanged_nudge() {
        let mut lua = SiftLua::new(None, test_context()).unwrap();
        let plugin_code = r#"
            return {
                name = "test-cmd",
                priority = 0,
                pattern = "test-cmd",
                execute = function(ctx, args, stdin)
                    sift.nudge(ctx, "bypass: 'command cat foo.rs'")
                    return { status = "unchanged", message = "[sift] foo.rs unchanged since last read" }
                end
            }
        "#;
        lua.load_plugin_from_str("test", plugin_code).unwrap();
        let (output, exit_code, _plugin) = lua.dispatch("test-cmd", &[], None).unwrap();
        assert!(output.contains("[sift] foo.rs unchanged since last read"), "output: {output}");
        assert!(output.contains("bypass: 'command cat foo.rs'"), "output: {output}");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_cleanup_cache() {
        let session_id = "test-cleanup";
        let base = std::path::PathBuf::from("/tmp/sift").join(session_id);
        let cache_dir = base.join("cache");
        let objects_dir = base.join("objects");
        let _ = std::fs::remove_dir_all(&base);

        // Create a fresh cache entry
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::create_dir_all(&objects_dir).unwrap();
        let meta = serde_json::json!({"created_at": 1_000_000_000_000u64, "size": 10});
        std::fs::write(cache_dir.join("abc123"), meta.to_string()).unwrap();
        std::fs::write(objects_dir.join("sha256-abc123.txt"), "content").unwrap();

        // Create an orphan object (no cache entry)
        std::fs::write(objects_dir.join("sha256-orphan.txt"), "orphan").unwrap();

        // Run cleanup with very short TTL (1ms) — should delete everything
        super::cleanup_cache(session_id, 1);

        // Cache entry should be deleted (expired)
        assert!(!cache_dir.join("abc123").exists(), "expired cache entry should be deleted");
        // Orphan object should be deleted
        assert!(!objects_dir.join("sha256-orphan.txt").exists(), "orphan object should be deleted");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_cleanup_cache_preserves_fresh() {
        let session_id = "test-cleanup-fresh";
        let base = std::path::PathBuf::from("/tmp/sift").join(session_id);
        let cache_dir = base.join("cache");
        let objects_dir = base.join("objects");
        let _ = std::fs::remove_dir_all(&base);

        // Create a fresh cache entry (recent timestamp)
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::create_dir_all(&objects_dir).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let meta = serde_json::json!({"created_at": now, "size": 10});
        std::fs::write(cache_dir.join("abc123"), meta.to_string()).unwrap();
        std::fs::write(objects_dir.join("sha256-abc123.txt"), "content").unwrap();

        // Run cleanup with long TTL — should preserve everything
        super::cleanup_cache(session_id, 86_400_000);

        assert!(cache_dir.join("abc123").exists(), "fresh cache entry should be preserved");
        assert!(objects_dir.join("sha256-abc123.txt").exists(), "referenced object should be preserved");

        let _ = std::fs::remove_dir_all(&base);
    }
}
