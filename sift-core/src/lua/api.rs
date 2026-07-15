use super::{exec_command, find_real_bash, save_output, PluginEntry, SiftLua};
use crate::classifier::classify;
use anyhow::{Context, Result};
use jaq_interpret::{FilterT, RcIter};
use mlua::{Function, Lua, LuaSerdeExt, Table, Value};
use sha2::Digest;

impl SiftLua {
    pub(crate) fn register_sift_table(&self) -> Result<()> {
        let sift = self.lua.create_table()?;
        self.register_exec(&sift)?;
        self.register_cache(&sift)?;
        self.register_hash(&sift)?;
        self.register_fs(&sift)?;
        self.register_json_toon(&sift)?;
        self.register_jq(&sift)?;
        self.register_env(&sift)?;
        self.register_classify(&sift)?;
        self.register_store(&sift)?;
        self.register_meta(&sift)?;
        self.lua.globals().set("sift", sift)?;
        Ok(())
    }

    fn register_exec(&self, sift: &Table) -> Result<()> {
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count;
        let nudges = self.nudges.clone();
        let exec_fn = self.lua.create_function(move |_, (_ctx, cmd): (Table, String)| {
            let (stdout, stderr, exit_code) = exec_command(&cmd, &session_id, cmd_count)?;
            let combined = format!("{stdout}{stderr}");
            // On-error save with auto-nudge
            if exit_code != 0 {
                let path = save_output(&cmd, &session_id, cmd_count, &combined);
                if let Ok(mut guard) = nudges.lock() {
                    guard.push(format!("use 'command cat {path}' for raw output"));
                }
            }
            Ok((combined, stderr, exit_code))
        })?;
        sift.set("exec", exec_fn)?;
        self.register_log(sift)?;

        let exit_fn = self.lua.create_function(|_, (_ctx, code): (Table, i32)| -> mlua::Result<()> {
            std::process::exit(code);
        })?;
        sift.set("exit", exit_fn)?;

        let output_fn = self.lua.create_function(|_, (_ctx, text): (Table, String)| {
            print!("{text}");
            Ok(())
        })?;
        sift.set("output", output_fn)?;
        Ok(())
    }

    fn register_log(&self, sift: &Table) -> Result<()> {
        let log_table = self.lua.create_table()?;
        let log_fn = self.lua.create_function(|_, (_ctx, level, msg): (Table, String, String)| {
            match level.as_str() {
                "error" => eprintln!("[sift] ERROR: {msg}"),
                "warn" => eprintln!("[sift] WARN: {msg}"),
                "info" => println!("[sift] INFO: {msg}"),
                "debug" => println!("[sift] DEBUG: {msg}"),
                _ => eprintln!("[sift] {level}: {msg}"),
            }
            Ok(())
        })?;
        let log_metatable = self.lua.create_table()?;
        log_metatable.set("__call", log_fn)?;
        log_table.set_metatable(Some(log_metatable));

        // sift.log.nudge(ctx, msg)
        let nudges = self.nudges.clone();
        let nudge_fn = self.lua.create_function(move |_, (_ctx, msg): (Table, String)| {
            if let Ok(mut guard) = nudges.lock() {
                guard.push(msg);
            }
            Ok(())
        })?;
        log_table.set("nudge", nudge_fn)?;
        sift.set("log", log_table)?;
        Ok(())
    }

    fn register_cache(&self, sift: &Table) -> Result<()> {
        let cache = self.lua.create_table()?;
        let store: Option<std::sync::Arc<crate::session::SessionStore>> = self.store.clone();

        // sift.cache.has(ctx, key) -> bool
        let f_has = self.lua.create_function(move |_, (ctx, key): (Table, String)| {
            let session_id: String = ctx.get("session_id")?;
            store.as_ref().map_or_else(|| Ok(false), |s| {
                match futures::executor::block_on(s.cache_has(&key, &session_id)) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(mlua::Error::external(e.to_string())),
                }
            })
        })?;
        cache.set("has", f_has)?;

        // sift.cache.set(ctx, key)
        let store2 = self.store.clone();
        let f_set = self.lua.create_function(move |_, (ctx, key): (Table, String)| {
            let session_id: String = ctx.get("session_id")?;
            if let Some(ref store) = store2 {
                futures::executor::block_on(store.cache_set(&key, &session_id))
                    .map_err(|e| mlua::Error::external(e.to_string()))?;
            }
            Ok(())
        })?;
        cache.set("set", f_set)?;

        // sift.cache.reset(ctx)
        let store3 = self.store.clone();
        let f_reset = self.lua.create_function(move |_, ctx: Table| {
            let session_id: String = ctx.get("session_id")?;
            if let Some(ref store) = store3 {
                futures::executor::block_on(store.cache_reset(&session_id))
                    .map_err(|e| mlua::Error::external(e.to_string()))?;
            }
            Ok(())
        })?;
        cache.set("reset", f_reset)?;

        sift.set("cache", cache)?;
        Ok(())
    }

    fn register_hash(&self, sift: &Table) -> Result<()> {
        let hash = self.lua.create_table()?;
        let sha256_fn = self.lua.create_function(|_, (ctx, data): (Table, String)| {
            let _ = ctx; // ctx unused, accepted for API consistency
            Ok(hex::encode(sha2::Sha256::digest(data.as_bytes())))
        })?;
        hash.set("sha256", sha256_fn)?;
        let md5_fn = self.lua.create_function(|_, (ctx, data): (Table, String)| {
            let _ = ctx; // ctx unused, accepted for API consistency
            Ok(hex::encode(md5::Md5::digest(data.as_bytes())))
        })?;
        hash.set("md5", md5_fn)?;
        sift.set("hash", hash)?;
        Ok(())
    }

    fn register_fs(&self, sift: &Table) -> Result<()> {
        let fs = self.lua.create_table()?;
        let fs_read = self.lua.create_function(|_, (ctx, path, opts): (Table, String, Option<Table>)| {
            let _ = ctx;
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
        let fs_write = self.lua.create_function(|_, (ctx, path, content): (Table, String, String)| {
            let _ = ctx;
            std::fs::write(&path, &content)
                .map_err(|e| mlua::Error::external(format!("write {path}: {e}")))?;
            Ok(())
        })?;
        fs.set("write", fs_write)?;

        // fs.edit(path, edits) — apply multiple disjoint text replacements
        let fs_edit = self.lua.create_function(|_, (ctx, path, edits): (Table, String, Table)| {
            let _ = ctx;
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
        let fs_stat = self.lua.create_function(|lua, (ctx, path): (Table, String)| {
            let _ = ctx;
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
        let fs_exists = self.lua.create_function(|_, (ctx, path): (Table, String)| {
            let _ = ctx;
            Ok(std::path::Path::new(&path).exists())
        })?;
        fs.set("exists", fs_exists)?;

        sift.set("fs", fs)?;
        Ok(())
    }

    fn register_json_toon(&self, sift: &Table) -> Result<()> {
        let json = self.lua.create_table()?;
        let json_encode = self.lua.create_function(|lua, (ctx, val): (Table, Value)| {
            let _ = ctx;
            let json_val = lua.from_value::<serde_json::Value>(val)
                .map_err(|e| mlua::Error::external(format!("json encode: {e}")))?;
            serde_json::to_string(&json_val)
                .map_err(|e| mlua::Error::external(format!("json encode: {e}")))
        })?;
        json.set("encode", json_encode)?;
        let json_decode = self.lua.create_function(|lua, (ctx, s): (Table, String)| {
            let _ = ctx;
            let json_val: serde_json::Value = serde_json::from_str(&s)
                .map_err(|e| mlua::Error::external(format!("json decode: {e}")))?;
            lua.to_value(&json_val)
                .map_err(|e| mlua::Error::external(format!("json decode: {e}")))
        })?;
        json.set("decode", json_decode)?;

        // sift.json.shortest(ctx, raw, formats) — token-aware JSON optimization
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count;
        let nudges = self.nudges.clone();
        let shortest_fn = self.lua.create_function(move |_lua, (ctx, raw, formats): (Table, String, Table)| {
            let _ = ctx;
            // Parse raw JSON — if invalid, return raw unchanged
            let json_val: serde_json::Value = match serde_json::from_str(&raw) {
                Ok(v) => v,
                Err(_) => return Ok(raw),
            };

            // Always include compacted JSON as baseline (no pretty-print)
            let compact_raw = serde_json::to_string(&json_val).unwrap_or_else(|_| raw.clone());
            let mut candidates: Vec<(String, String)> = Vec::new(); // (format_name, output)
            candidates.push(("raw".to_string(), compact_raw));

            // Check if json format is requested
            if let Some(json_opts) = formats.get::<Option<Table>>("json").ok().flatten() {
                let max_string_len: usize = json_opts.get("max_string_len").unwrap_or(80);
                let max_array_items: usize = json_opts.get("max_array_items").unwrap_or(10);
                let max_depth: usize = json_opts.get("max_depth").unwrap_or(5);
                let max_keys: usize = json_opts.get("max_keys").unwrap_or(20);
                let compacted = compact_json(&json_val, max_string_len, max_array_items, max_depth, max_keys);
                candidates.push(("json".to_string(), compacted));
            }

            // Check if toon format is requested
            let has_toon = formats.get::<bool>("toon").unwrap_or(false)
                || formats.get::<Option<Table>>("toon").ok().flatten().is_some();
            if has_toon {
                if let Ok(toon_output) = toon_format::encode_default(&json_val) {
                    candidates.push(("toon".to_string(), toon_output));
                }
            }

            // Token cost: rough estimate (len/4)
            // Compute nudge overhead dynamically: "\n[sift] raw: 'command cat <path>'"
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(&session_id);
            let nudge_path = tmp_dir.join(format!("{ts}_{cmd_count}_raw_original.json"));
            let nudge_path_str = nudge_path.display().to_string();
            // Nudge format: "\n[sift] raw: 'command cat {path}'" = 8 + 20 + path_len chars
            let nudge_msg_len = 8 + 20 + nudge_path_str.len();
            let nudge_overhead = nudge_msg_len / 4;
            let mut best_idx = 0usize;
            let mut best_cost = candidates[0].1.len() / 4;

            for (i, (_name, output)) in candidates.iter().enumerate().skip(1) {
                let cost = output.len() / 4 + nudge_overhead;
                if cost < best_cost {
                    best_cost = cost;
                    best_idx = i;
                }
            }

            let (best_name, best_output) = &candidates[best_idx];

            // If non-raw format wins, store raw and emit nudge
            if best_name != "raw" {
                let _ = std::fs::create_dir_all(&tmp_dir);
                let _ = std::fs::write(&nudge_path, &raw);
                if let Ok(mut guard) = nudges.lock() {
                    guard.push(format!("raw: 'command cat {nudge_path_str}'"));
                }
            }

            Ok(best_output.clone())
        })?;
        json.set("shortest", shortest_fn)?;

        sift.set("json", json)?;

        let toon = self.lua.create_table()?;
        let toon_encode = self.lua.create_function(|lua, (ctx, val): (Table, Value)| {
            let _ = ctx;
            let json_val = lua.from_value::<serde_json::Value>(val)
                .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))?;
            toon_format::encode_default(&json_val)
                .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))
        })?;
        toon.set("encode", toon_encode)?;
        let toon_decode = self.lua.create_function(|lua, (ctx, s): (Table, String)| {
            let _ = ctx;
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
        let jq_query = self.lua.create_function(|lua, (ctx, data, filter): (Table, Value, String)| {
            let _ = ctx;
            let json_val: serde_json::Value = if let Value::String(s) = &data {
                let s = s.to_str().map_err(|e| mlua::Error::external(format!("jq str: {e}")))?;
                serde_json::from_str(&s)
                    .map_err(|e| mlua::Error::external(format!("jq parse: {e}")))?
            } else {
                lua.from_value(data)
                    .map_err(|e| mlua::Error::external(format!("jq convert: {e}")))?
            };
            let (f, errs) = jaq_parse::parse(&filter, jaq_parse::main());
            if !errs.is_empty() {
                return Err(mlua::Error::external(format!("jq parse errors: {errs:?}")));
            }
            let f = f.ok_or_else(|| mlua::Error::external("jq: no filter".to_string()))?;
            let mut ctx = jaq_interpret::ParseCtx::new(Vec::new());
            let filter = ctx.compile(f);
            let inputs = RcIter::new(core::iter::empty());
            let cv = (jaq_interpret::Ctx::new([], &inputs), jaq_interpret::Val::from(json_val));
            let mut outputs = Vec::new();
            for val in filter.run(cv) {
                match val {
                    Ok(v) => {
                        let s = format!("{v}");
                        if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&s) {
                            outputs.push(jv);
                        }
                    },
                    Err(e) => return Err(mlua::Error::external(format!("jq: {e}"))),
                }
            }
            serde_json::to_string(&outputs)
                .map_err(|e| mlua::Error::external(format!("jq result: {e}")))
        })?;
        jq.set("query", jq_query)?;
        sift.set("jq", jq)?;
        Ok(())
    }

    fn register_env(&self, sift: &Table) -> Result<()> {
        let env = self.lua.create_table()?;
        let env_get = self.lua.create_function(|_, (ctx, key): (Table, String)| {
            let _ = ctx;
            Ok(std::env::var(key).ok())
        })?;
        env.set("get", env_get)?;
        let env_set = self.lua.create_function(|_, (ctx, key, val): (Table, String, String)| {
            let _ = ctx;
            std::env::set_var(&key, &val);
            Ok(())
        })?;
        env.set("set", env_set)?;
        sift.set("env", env)?;
        Ok(())
    }

    fn register_classify(&self, sift: &Table) -> Result<()> {
        let classify_fn = self.lua.create_function(|_, (ctx, cmd): (Table, String)| {
            let _ = ctx;
            let result = classify(&cmd);
            let lua = Lua::new();
            let tbl = lua.create_table()?;
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

        let token_count_fn = self.lua.create_function(|_, (_ctx, text): (Table, String)| {
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

    fn register_store(&self, sift: &Table) -> Result<()> {
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count;
        let nudges = self.nudges.clone();
        let store_fn = self.lua.create_function(move |_, (_ctx, content, slug): (Table, String, String)| {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(&session_id);
            let _ = std::fs::create_dir_all(&tmp_dir);
            let safe_slug: String = slug.chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
                .collect();
            let path = tmp_dir.join(format!("{ts}_{cmd_count}_{safe_slug}"));
            let path_str = path.display().to_string();
            let _ = std::fs::write(&path, &content);
            if let Ok(mut guard) = nudges.lock() {
                guard.push(format!("stored: 'command cat {path_str}'"));
            }
            Ok(path_str)
        })?;
        sift.set("store", store_fn)?;
        Ok(())
    }

    /// Load a plugin from a Lua string.
    ///
    /// The Lua code should return a table with:
    /// - `name`: string — command name
    /// - `priority`: number (optional, default 0)
    /// - `pattern`: string | string[] (optional, defaults to name)
    /// - `execute`: function(ctx, args, stdin) -> result table
    pub fn load_plugin_from_str(&mut self, name: &str, lua_code: &str) -> Result<()> {
        let chunk = self.lua.load(lua_code).set_name(name);
        let plugin_table: Table = chunk.eval().with_context(|| format!("failed to load plugin {name}"))?;

        let plugin_name: String = plugin_table.get("name")?;
        let priority: i32 = plugin_table.get("priority").unwrap_or(0);

        // Support pattern as string or string[]
        let patterns: Vec<String> = plugin_table.get::<String>("pattern")
            .map(|s| vec![s])
            .or_else(|_| plugin_table.get::<Vec<String>>("pattern"))
            .unwrap_or_else(|_| vec![plugin_name]);

        // Store the plugin table reference
        let key = self.lua.create_registry_value(plugin_table)?;

        self.plugins.push(PluginEntry {
            patterns,
            priority,
            table: key,
        });

        // Sort by: longest pattern first (use first pattern for sorting), then higher priority
        self.plugins.sort_by(|a, b| {
            let a_max = a.patterns.iter().map(String::len).max().unwrap_or(0);
            let b_max = b.patterns.iter().map(String::len).max().unwrap_or(0);
            b_max.cmp(&a_max)
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
            if let Some(entry) = self.plugins.iter().find(|e| e.patterns.iter().any(|p| p == candidate)) {
                return Some(entry);
            }
        }

        None
    }

    /// Dispatch a full command string, handling cd/pushd/popd prefixes and pipelines.
    ///
    /// Returns `(output, exit_code, plugin_name)`.
    ///
    /// # Panics
    ///
    /// Panics if the Lua dispatch encounters an unexpected error.
    pub fn dispatch_full(&self, full_cmd: &str, stdin: Option<&str>) -> Result<(String, i32, String)> {
        // Handle cd <dir> && <command> — peel cd prefix, chdir, dispatch rest
        if let Some(rest) = peel_cd_prefix(full_cmd) {
            return self.dispatch_full(&rest, stdin);
        }

        // Handle pushd <dir> && <command>
        if let Some(rest) = peel_pushd_prefix(full_cmd) {
            return self.dispatch_full(&rest, stdin);
        }

        // Handle popd
        if full_cmd.trim() == "popd" {
            let prev = std::env::current_dir()
                .ok()
                .and_then(|c| c.parent().map(std::path::Path::to_path_buf));
            if let Some(dir) = prev {
                let _ = std::env::set_current_dir(&dir);
            }
            return Ok((String::new(), 0, "cd".to_string()));
        }

        // Pipeline optimization: check if command contains pipes
        let trimmed = full_cmd.trim();
        if trimmed.contains('|') && !trimmed.contains("||") {
            // Split by | (not ||)
            let segments: Vec<&str> = split_pipeline(trimmed);
            if segments.len() > 1 {
                let last_segment = segments.last().unwrap().trim();
                let last_parts: Vec<&str> = last_segment.split_whitespace().collect();
                if !last_parts.is_empty() {
                    let last_name = last_parts[0];
                    let last_args: Vec<String> = last_parts[1..].iter().map(ToString::to_string).collect();

                    // Check if last command matches a plugin
                    if self.find_plugin(last_name, &last_args).is_some() {
                        // Run preceding segments in bash, pipe to plugin
                        let preceding = segments[..segments.len() - 1].join(" | ");
                        let bash_path = find_real_bash();
                        let output = std::process::Command::new(&bash_path)
                            .arg("-c")
                            .arg(&preceding)
                            .env("PAGER", "cat")
                            .env("TERM", "dumb")
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .output()
                            .map_err(|e| mlua::Error::external(format!("pipeline spawn: {e}")))?;
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let exit_code = output.status.code().unwrap_or(1);

                        if exit_code != 0 {
                            return Ok((format!("{stdout}{stderr}"), exit_code, "pipeline".to_string()));
                        }

                        // Dispatch last segment to plugin with preceding stdout as stdin
                        return self.dispatch(last_name, &last_args, Some(&stdout));
                    }
                }
            }
        }

        // Normal dispatch
        let parts: Vec<&str> = full_cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Ok((String::new(), 0, String::new()));
        }
        let name = parts[0];
        let args: Vec<String> = parts[1..].iter().map(ToString::to_string).collect();
        self.dispatch(name, &args, stdin)
    }

    /// Dispatch a command to the best matching plugin.
    ///
    /// Returns `(output, exit_code, plugin_name)`.
    pub fn dispatch(&self, cmd: &str, args: &[String], stdin: Option<&str>) -> Result<(String, i32, String)> {
        let entry = self.find_entry(cmd, args)?;

        let plugin_table: Table = self.lua.registry_value(&entry.table)?;
        let execute: Function = plugin_table.get("execute")?;

        // Build context table
        let ctx = self.lua.create_table()?;
        ctx.set("cwd", self.ctx.cwd.display().to_string())?;
        ctx.set("cmd_count", self.ctx.cmd_count)?;
        ctx.set("session_id", self.ctx.session_id.clone().unwrap_or_default())?;
        ctx.set("command", cmd)?;

        // Build args table (arguments only, no command name)
        let args_table = self.lua.create_table()?;
        for (i, arg) in args.iter().enumerate() {
            args_table.set(i + 1, arg.clone())?;
        }

        let stdin_val = match stdin {
            Some(s) => Value::String(self.lua.create_string(s)?),
            None => Value::Nil,
        };

        // Clear nudges from previous dispatch
        if let Ok(mut guard) = self.nudges.lock() {
            guard.clear();
        }

        let result: Table = execute.call((ctx, args_table, stdin_val))?;

        let status: String = result.get("status")?;
        let output: String = result.get("output").unwrap_or_default();
        let exit_code: i32 = result.get("exit_code").unwrap_or(0);

        if status == "passthrough" {
            return Self::execute_passthrough(cmd, args);
        }

        let final_output = if status == "unchanged" {
            // Emit auto-nudge for bypass hint
            if let Ok(mut guard) = self.nudges.lock() {
                guard.push(format!("use 'command cat {}' for unfiltered content", result.get::<String>("path").unwrap_or_default()));
            }
            result.get::<String>("message").unwrap_or(output)
        } else {
            output
        };

        let nudge_text = self.collect_nudges();
        let final_output = if nudge_text.is_empty() {
            final_output
        } else {
            format!("{final_output}{nudge_text}")
        };

        Ok((final_output, exit_code, entry.patterns.first().cloned().unwrap_or_default()))
    }

    /// Find the best matching plugin entry, falling back to __default__.
    fn find_entry(&self, cmd: &str, args: &[String]) -> Result<&PluginEntry> {
        self.find_plugin(cmd, args).map_or_else(
            || {
                self.plugins
                    .iter()
                    .find(|e| e.patterns.iter().any(|p| p == "__default__"))
                    .ok_or_else(|| anyhow::anyhow!("no matching plugin and no __default__"))
            },
            Ok,
        )
    }

    /// Collect accumulated nudges into a formatted string, clearing the buffer.
    fn collect_nudges(&self) -> String {
        let nudges = match self.nudges.lock() {
            Ok(mut guard) => guard.drain(..).collect::<Vec<_>>(),
            Err(_) => return String::new(),
        };
        if nudges.is_empty() {
            return String::new();
        }
        let mut text = String::new();
        for n in &nudges {
            use std::fmt::Write;
            let _ = write!(text, "\n[sift] {n}");
        }
        text
    }

    /// Execute a command directly (passthrough — bypass all plugins).
    fn execute_passthrough(cmd: &str, args: &[String]) -> Result<(String, i32, String)> {
        let full_cmd = if args.is_empty() {
            cmd.to_string()
        } else {
            format!("{} {}", cmd, args.join(" "))
        };
        // Use exec_command to execute the command directly
        let (stdout, stderr, exit_code) = exec_command(&full_cmd, "", 0)?;
        let combined = format!("{stdout}{stderr}");
        Ok((combined, exit_code, "passthrough".to_string()))
    }
}

/// Split a command string by `|` pipe separators, but not `||`.
pub fn split_pipeline(input: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '|' && i + 1 < len && chars[i + 1] == '|' {
            // Skip || (logical OR)
            i += 2;
        } else if chars[i] == '|' {
            segments.push(input[start..i].trim());
            start = i + 1;
            i += 1;
        } else {
            i += 1;
        }
    }
    if start < len {
        segments.push(input[start..].trim());
    }
    segments
}

/// If `input` starts with `cd <dir> && ` or `cd <dir> ; `, change directory and return the rest.
fn peel_cd_prefix(input: &str) -> Option<String> {
    let trimmed = input.trim();
    // Match `cd <dir> && <rest>` or `cd <dir> ; <rest>`
    let re = regex_lite::Regex::new(r"^cd\s+(.+?)\s*(?:&&|;)\s*(.+)$").ok()?;
    if let Some(caps) = re.captures(trimmed) {
        let dir = caps.get(1)?.as_str().trim();
        let rest = caps.get(2)?.as_str().trim();
        // Change directory
        let _ = std::env::set_current_dir(dir);
        Some(rest.to_string())
    } else {
        None
    }
}

/// If `input` starts with `pushd <dir> && `, change directory and return the rest.
fn peel_pushd_prefix(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let re = regex_lite::Regex::new(r"^pushd\s+(.+?)\s*&&\s*(.+)$").ok()?;
    if let Some(caps) = re.captures(trimmed) {
        let dir = caps.get(1)?.as_str().trim();
        let rest = caps.get(2)?.as_str().trim();
        let _ = std::env::set_current_dir(dir);
        Some(rest.to_string())
    } else {
        None
    }
}

/// Compact a JSON value: truncate long strings, summarize large arrays, limit depth/keys.
fn compact_json(
    val: &serde_json::Value,
    max_string_len: usize,
    max_array_items: usize,
    max_depth: usize,
    max_keys: usize,
) -> String {
    let compacted = compact_value(val, max_string_len, max_array_items, max_depth, max_keys, 0);
    serde_json::to_string(&compacted).unwrap_or_default()
}

fn compact_value(
    val: &serde_json::Value,
    max_string_len: usize,
    max_array_items: usize,
    max_depth: usize,
    max_keys: usize,
    depth: usize,
) -> serde_json::Value {
    if depth > max_depth {
        return serde_json::Value::String("...".to_string());
    }
    match val {
        serde_json::Value::String(s) => {
            if s.len() > max_string_len {
                let truncated: String = s.chars().take(max_string_len).collect();
                serde_json::Value::String(format!("{truncated}..."))
            } else {
                serde_json::Value::String(s.clone())
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.len() > max_array_items {
                let mut items: Vec<serde_json::Value> = arr[..max_array_items]
                    .iter()
                    .map(|v| compact_value(v, max_string_len, max_array_items, max_depth, max_keys, depth + 1))
                    .collect();
                let remaining = arr.len() - max_array_items;
                items.push(serde_json::Value::String(format!("... +{remaining} more")));
                serde_json::Value::Array(items)
            } else {
                serde_json::Value::Array(
                    arr.iter()
                        .map(|v| compact_value(v, max_string_len, max_array_items, max_depth, max_keys, depth + 1))
                        .collect(),
                )
            }
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<(String, serde_json::Value)> = obj
                .iter()
                .take(max_keys)
                .map(|(k, v)| {
                    (k.clone(), compact_value(v, max_string_len, max_array_items, max_depth, max_keys, depth + 1))
                })
                .collect();
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k, v);
            }
            if obj.len() > max_keys {
                map.insert(
                    "...".to_string(),
                    serde_json::Value::String(format!("+{} more keys", obj.len() - max_keys)),
                );
            }
            serde_json::Value::Object(map)
        }
        other => other.clone(),
    }
}

impl Drop for SiftLua {
    fn drop(&mut self) {
        // Registry keys are dropped when the Lua VM is dropped
    }
}
