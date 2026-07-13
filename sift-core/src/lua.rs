//! Lua runtime and `sift.*` API for plugins.
//!
//! Provides the mlua-based Lua VM, registers all `sift.*` functions,
//! and handles plugin loading and dispatch.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use mlua::{Function, Lua, LuaSerdeExt, Table, Value};
use sha2::Digest;

use crate::classifier::classify;
use crate::session::SessionStore;

/// Find the real bash binary, excluding our own path.
fn find_real_bash() -> PathBuf {
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

/// Execute a command via PTY and return `(output_string, exit_code)`.
fn exec_command(cmd: &str, session_id: &str, cmd_count: u64) -> Result<(String, i32), mlua::Error> {
    let (output_str, exit_code) = run_pty(cmd)?;
    save_output(cmd, session_id, cmd_count, &output_str);
    Ok((output_str, exit_code))
}

/// Run a command in a PTY and return the output and exit code.
fn run_pty(cmd: &str) -> Result<(String, i32), mlua::Error> {
    let bash_path = find_real_bash();
    let pty_system = portable_pty::native_pty_system();
    let pair = pty_system
        .openpty(portable_pty::PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| mlua::Error::external(format!("pty: {e}")))?;
    let cmd_builder = portable_pty::CommandBuilder::new(&bash_path);
    let mut child = pair.slave.spawn_command(cmd_builder)
        .map_err(|e| mlua::Error::external(format!("spawn: {e}")))?;
    let mut writer = pair.master.take_writer()
        .map_err(|e| mlua::Error::external(format!("writer: {e}")))?;
    let full_cmd = format!("{cmd}; exit $?\n");
    let _ = writer.write_all(full_cmd.as_bytes());
    let _ = writer.flush();
    drop(writer);
    let mut reader = pair.master.try_clone_reader()
        .map_err(|e| mlua::Error::external(format!("reader: {e}")))?;
    let mut output = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => output.extend_from_slice(&buf[..n]),
        }
    }
    let exit_code = child.wait()
        .map_or(1, |s| s.exit_code().cast_signed());
    let output_str = String::from_utf8_lossy(&output).to_string();
    Ok((output_str, exit_code))
}

/// Save raw output to a temp file.
fn save_output(cmd: &str, session_id: &str, cmd_count: u64, output: &str) {
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
struct PluginEntry {
    /// Command pattern for matching (e.g., "cat", "docker ps").
    pattern: String,
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
        };

        runtime.register_sift_table()?;
        Ok(runtime)
    }

    /// Register the `sift.*` API table in the Lua VM.
    fn register_sift_table(&self) -> Result<()> {
        let sift = self.lua.create_table()?;
        self.register_exec(&sift)?;
        self.register_cache(&sift)?;
        self.register_hash(&sift)?;
        self.register_fs(&sift)?;
        self.register_json_toon(&sift)?;
        self.register_jq(&sift)?;
        self.register_env(&sift)?;
        self.register_classify(&sift)?;
        self.register_meta(&sift)?;
        self.lua.globals().set("sift", sift)?;
        Ok(())
    }

    fn register_exec(&self, sift: &Table) -> Result<()> {
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count;
        let exec_fn = self.lua.create_function(move |_, cmd: String| {
            exec_command(&cmd, &session_id, cmd_count)
        })?;
        sift.set("exec", exec_fn)?;

        let log_fn = self.lua.create_function(|_, (level, msg): (String, String)| {
            match level.as_str() {
                "error" => eprintln!("[sift] ERROR: {msg}"),
                "warn" => eprintln!("[sift] WARN: {msg}"),
                "info" => println!("[sift] INFO: {msg}"),
                "debug" => println!("[sift] DEBUG: {msg}"),
                _ => eprintln!("[sift] {level}: {msg}"),
            }
            Ok(())
        })?;
        sift.set("log", log_fn)?;

        let exit_fn = self.lua.create_function(|_, code: i32| -> mlua::Result<()> {
            std::process::exit(code);
        })?;
        sift.set("exit", exit_fn)?;

        let output_fn = self.lua.create_function(|_, text: String| {
            print!("{text}");
            Ok(())
        })?;
        sift.set("output", output_fn)?;
        Ok(())
    }

    fn register_cache(&self, sift: &Table) -> Result<()> {
        let cache = self.lua.create_table()?;
        let cache_get = self.lua.create_function(|_, _key: String| {
            Ok(Value::Nil)
        })?;
        cache.set("get", cache_get)?;
        sift.set("cache", cache)?;
        Ok(())
    }

    fn register_hash(&self, sift: &Table) -> Result<()> {
        let hash = self.lua.create_table()?;
        let sha256_fn = self.lua.create_function(|_, data: String| {
            Ok(hex::encode(sha2::Sha256::digest(data.as_bytes())))
        })?;
        hash.set("sha256", sha256_fn)?;
        let md5_fn = self.lua.create_function(|_, data: String| {
            Ok(hex::encode(md5::Md5::digest(data.as_bytes())))
        })?;
        hash.set("md5", md5_fn)?;
        sift.set("hash", hash)?;
        Ok(())
    }

    fn register_fs(&self, sift: &Table) -> Result<()> {
        let fs = self.lua.create_table()?;
        let fs_read = self.lua.create_function(|_, (path, opts): (String, Option<Table>)| {
            let offset: Option<usize> = opts.as_ref().and_then(|t| t.get("offset").ok());
            let limit: Option<usize> = opts.as_ref().and_then(|t| t.get("limit").ok());
            let content = std::fs::read_to_string(&path)
                .map_err(|e| mlua::Error::external(format!("read {path}: {e}")))?;
            let lines: Vec<&str> = content.lines().collect();
            let start = offset.unwrap_or(1).saturating_sub(1);
            let end = limit.map_or(lines.len(), |l| start + l);
            let selected: Vec<&str> = lines.iter().skip(start).take(end.saturating_sub(start)).copied().collect();
            Ok(selected.join("\n"))
        })?;
        fs.set("read", fs_read)?;

        // fs.write(path, content)
        let fs_write = self.lua.create_function(|_, (path, content): (String, String)| {
            std::fs::write(&path, &content)
                .map_err(|e| mlua::Error::external(format!("write {path}: {e}")))?;
            Ok(())
        })?;
        fs.set("write", fs_write)?;

        // fs.edit(path, edits) — apply multiple disjoint text replacements
        let fs_edit = self.lua.create_function(|_, (path, edits): (String, Table)| {
            let mut content = std::fs::read_to_string(&path)
                .map_err(|e| mlua::Error::external(format!("read {path}: {e}")))?;
            let num_edits = usize::try_from(edits.len().map_err(|e| mlua::Error::external(e.to_string()))?)
                .map_err(|e| mlua::Error::external(format!("invalid edit count: {e}")))?;
            for i in 1..=num_edits {
                let edit: Table = edits.get(i)?;
                let old_text: String = edit.get("oldText")?;
                let new_text: String = edit.get("newText")?;
                if !content.contains(&old_text) {
                    return Err(mlua::Error::external(format!(
                        "edit {path}: oldText not found: {old_text}"
                    )));
                }
                content = content.replacen(&old_text, &new_text, 1);
            }
            std::fs::write(&path, &content)
                .map_err(|e| mlua::Error::external(format!("write {path}: {e}")))?;
            Ok(())
        })?;
        fs.set("edit", fs_edit)?;

        // fs.stat(path)
        let fs_stat = self.lua.create_function(|lua, path: String| {
            let meta = std::fs::metadata(&path)
                .map_err(|e| mlua::Error::external(format!("stat {path}: {e}")))?;
            let result = lua.create_table()?;
            result.set("size", meta.len())?;
            result.set("is_dir", meta.is_dir())?;
            result.set("is_file", meta.is_file())?;
            Ok(result)
        })?;
        fs.set("stat", fs_stat)?;

        // fs.exists(path)
        let fs_exists = self.lua.create_function(|_, path: String| {
            Ok(std::path::Path::new(&path).exists())
        })?;
        fs.set("exists", fs_exists)?;

        sift.set("fs", fs)?;
        Ok(())
    }

    fn register_json_toon(&self, sift: &Table) -> Result<()> {
        let json = self.lua.create_table()?;
        let json_encode = self.lua.create_function(|lua, val: Value| {
            let json_val = lua.from_value::<serde_json::Value>(val)
                .map_err(|e| mlua::Error::external(format!("json encode: {e}")))?;
            serde_json::to_string(&json_val)
                .map_err(|e| mlua::Error::external(format!("json encode: {e}")))
        })?;
        json.set("encode", json_encode)?;
        let json_decode = self.lua.create_function(|lua, s: String| {
            let json_val: serde_json::Value = serde_json::from_str(&s)
                .map_err(|e| mlua::Error::external(format!("json decode: {e}")))?;
            lua.to_value(&json_val)
                .map_err(|e| mlua::Error::external(format!("json decode: {e}")))
        })?;
        json.set("decode", json_decode)?;
        sift.set("json", json)?;

        let toon = self.lua.create_table()?;
        let toon_encode = self.lua.create_function(|lua, val: Value| {
            let json_val = lua.from_value::<serde_json::Value>(val)
                .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))?;
            toon_format::encode_default(&json_val)
                .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))
        })?;
        toon.set("encode", toon_encode)?;
        let toon_decode = self.lua.create_function(|lua, s: String| {
            let json_val: serde_json::Value = toon_format::decode_default(&s)
                .map_err(|e| mlua::Error::external(format!("toon decode: {e}")))?;
            lua.to_value(&json_val)
                .map_err(|e| mlua::Error::external(format!("toon decode: {e}")))
        })?;
        toon.set("decode", toon_decode)?;
        sift.set("toon", toon)?;
        Ok(())
    }

    fn register_jq(&self, sift: &Table) -> Result<()> {
        let jq = self.lua.create_table()?;
        let jq_query = self.lua.create_function(|lua, (data, filter): (Value, String)| {
            let json_str: String = if let Value::String(s) = &data {
                s.to_str()
                    .map_err(|e| mlua::Error::external(format!("jq str: {e}")))?
                    .to_string()
            } else {
                let json_val: serde_json::Value = lua.from_value(data)
                    .map_err(|e| mlua::Error::external(format!("jq convert: {e}")))?;
                serde_json::to_string(&json_val)
                    .map_err(|e| mlua::Error::external(format!("jq serialize: {e}")))?
            };
            let output = std::process::Command::new("jaq")
                .arg(&filter)
                .arg(json_str)
                .output()
                .map_err(|e| mlua::Error::external(format!("jq exec: {e}")))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(mlua::Error::external(format!("jq error: {stderr}")));
            }
            let result = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(result)
        })?;
        jq.set("query", jq_query)?;
        sift.set("jq", jq)?;
        Ok(())
    }

    fn register_env(&self, sift: &Table) -> Result<()> {
        let env = self.lua.create_table()?;
        let env_get = self.lua.create_function(|_, key: String| {
            Ok(std::env::var(key).ok())
        })?;
        env.set("get", env_get)?;
        let env_set = self.lua.create_function(|_, (key, val): (String, String)| {
            std::env::set_var(&key, &val);
            Ok(())
        })?;
        env.set("set", env_set)?;
        sift.set("env", env)?;
        Ok(())
    }

    fn register_classify(&self, sift: &Table) -> Result<()> {
        let classify_fn = self.lua.create_function(|_, cmd: String| {
            let result = classify(&cmd);
            let lua = Lua::new();
            let tbl = lua.create_table()?;
            tbl.set("kind", format!("{:?}", result.kind))?;
            tbl.set("name", result.name)?;
            tbl.set("is_piped", result.is_piped)?;
            tbl.set("is_compound", result.is_compound)?;
            let args_tbl = lua.create_table()?;
            for (i, arg) in result.args.iter().enumerate() {
                args_tbl.set(i + 1, arg.clone())?;
            }
            tbl.set("args", args_tbl)?;
            Ok(tbl)
        })?;
        sift.set("classify", classify_fn)?;

        let token_count_fn = self.lua.create_function(|_, text: String| {
            Ok(i64::try_from(text.len() / 4).unwrap_or(i64::MAX))
        })?;
        sift.set("token_count", token_count_fn)?;
        Ok(())
    }

    fn register_meta(&self, sift: &Table) -> Result<()> {
        let meta = self.lua.create_table()?;
        let ctx = self.ctx.clone();
        meta.set("session_id", ctx.session_id.unwrap_or_default())?;
        meta.set("cmd_count", ctx.cmd_count)?;
        meta.set("cwd", ctx.cwd.display().to_string())?;
        meta.set("raw_bytes", ctx.raw_bytes)?;
        meta.set("filtered_bytes", ctx.filtered_bytes)?;
        sift.set("meta", meta)?;
        Ok(())
    }

    /// Load a plugin from a Lua string.
    ///
    /// The Lua code should return a table with:
    /// - `name`: string — command name
    /// - `priority`: number (optional, default 0)
    /// - `pattern`: string (optional, defaults to name)
    /// - `execute`: function(ctx, args, stdin) -> result table
    pub fn load_plugin_from_str(&mut self, name: &str, lua_code: &str) -> Result<()> {
        let chunk = self.lua.load(lua_code).set_name(name);
        let plugin_table: Table = chunk.eval().with_context(|| format!("failed to load plugin {name}"))?;

        let plugin_name: String = plugin_table.get("name")?;
        let priority: i32 = plugin_table.get("priority").unwrap_or(0);
        let pattern: String = plugin_table.get("pattern").unwrap_or_else(|_| plugin_name.clone());

        // Store the plugin table reference
        let key = self.lua.create_registry_value(plugin_table)?;

        self.plugins.push(PluginEntry {
            pattern,
            priority,
            table: key,
        });

        // Sort by: longer pattern first, then higher priority
        self.plugins.sort_by(|a, b| {
            b.pattern
                .len()
                .cmp(&a.pattern.len())
                .then_with(|| b.priority.cmp(&a.priority))
        });

        Ok(())
    }

    /// Find the best matching plugin for a command.
    fn find_plugin(&self, cmd: &str, args: &[String]) -> Option<&PluginEntry> {
        let mut candidates = vec![cmd.to_string()];
        let mut full = cmd.to_string();
        for arg in args {
            full.push(' ');
            full.push_str(arg);
            candidates.push(full.clone());
        }

        for candidate in candidates.iter().rev() {
            if let Some(entry) = self.plugins.iter().find(|e| e.pattern == *candidate) {
                return Some(entry);
            }
        }

        None
    }

    /// Dispatch a command to the best matching plugin.
    ///
    /// Returns `(output, exit_code, plugin_name)`.
    pub fn dispatch(&self, cmd: &str, args: &[String], stdin: Option<&str>) -> Result<(String, i32, String)> {
        let entry = match self.find_plugin(cmd, args) {
            Some(e) => e,
            None => {
                // No matching plugin found, try the default fallback
                match self.plugins.iter().find(|e| e.pattern == "__default__") {
                    Some(e) => e,
                    None => return Ok((String::new(), 0, "none".to_string())),
                }
            }
        };

        let plugin_table: Table = self.lua.registry_value(&entry.table)?;
        let execute: Function = plugin_table.get("execute")?;

        // Build context table
        let ctx = self.lua.create_table()?;
        ctx.set("cwd", self.ctx.cwd.display().to_string())?;
        ctx.set("cmd_count", self.ctx.cmd_count)?;
        ctx.set("session_id", self.ctx.session_id.clone().unwrap_or_default())?;

        // Build args table
        let args_table = self.lua.create_table()?;
        for (i, arg) in args.iter().enumerate() {
            args_table.set(i + 1, arg.clone())?;
        }

        let stdin_val = match stdin {
            Some(s) => Value::String(self.lua.create_string(s)?),
            None => Value::Nil,
        };

        let result: Table = execute.call((ctx, args_table, stdin_val))?;

        let _status: String = result.get("status")?;
        let output: String = result.get("output").unwrap_or_default();
        let exit_code: i32 = result.get("exit_code").unwrap_or(0);

        Ok((output, exit_code, entry.pattern.clone()))
    }
}

impl Drop for SiftLua {
    fn drop(&mut self) {
        // Registry keys are dropped when the Lua VM is dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_sift_lua_creation() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        assert!(sift.get::<mlua::Function>("exec").is_ok());
        assert!(sift.get::<mlua::Function>("log").is_ok());
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
        let result: String = sha256.call("hello").unwrap();
        assert_eq!(result, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_sift_token_count() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let token_count: mlua::Function = sift.get("token_count").unwrap();
        let result: isize = token_count.call("hello world").unwrap();
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
        let content: String = fs_read.call(("Cargo.toml", mlua::Value::Nil)).unwrap();
        assert!(content.contains("sift-core"));
    }

    #[test]
    fn test_sift_json_encode() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let json: Table = sift.get("json").unwrap();
        let encode: mlua::Function = json.get("encode").unwrap();
        let tbl = lua.lua.create_table().unwrap();
        tbl.set("name", "test").unwrap();
        let encoded: String = encode.call(tbl).unwrap();
        assert!(encoded.contains("\"name\""));
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
        fs_write.call::<()>((path.clone(), "hello world")).unwrap();
        let content: String = fs_read.call((path, mlua::Value::Nil)).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_sift_fs_stat() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let fs: Table = sift.get("fs").unwrap();
        let fs_stat: mlua::Function = fs.get("stat").unwrap();
        let result: Table = fs_stat.call("Cargo.toml").unwrap();
        let is_file: bool = result.get("is_file").unwrap();
        assert!(is_file);
    }

    #[test]
    fn test_sift_fs_exists() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let fs: Table = sift.get("fs").unwrap();
        let fs_exists: mlua::Function = fs.get("exists").unwrap();
        assert!(fs_exists.call::<bool>("Cargo.toml").unwrap());
        assert!(!fs_exists.call::<bool>("nonexistent_file_xyz").unwrap());
    }

    #[test]
    fn test_sift_toon_encode() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let toon: Table = sift.get("toon").unwrap();
        let encode: mlua::Function = toon.get("encode").unwrap();
        let tbl = lua.lua.create_table().unwrap();
        tbl.set("name", "test").unwrap();
        let encoded: String = encode.call(tbl).unwrap();
        assert!(encoded.contains("name"));
    }

    #[test]
    fn test_sift_env() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let env: Table = sift.get("env").unwrap();
        let env_set: mlua::Function = env.get("set").unwrap();
        let env_get: mlua::Function = env.get("get").unwrap();
        env_set.call::<()>(("SIFT_TEST", "val")).unwrap();
        let result: Option<String> = env_get.call("SIFT_TEST").unwrap();
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
        let (output, code) = exec_command("echo hello", "test", 0).unwrap();
        assert!(output.contains("hello"), "output should contain hello, got: {output}");
        assert_eq!(code, 0);
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
}
