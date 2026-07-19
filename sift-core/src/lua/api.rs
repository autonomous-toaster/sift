use super::exec::{exec_command, find_real_bash};
use super::stdin_reader::StdinReader;
use super::{PluginEntry, SiftLua};
use anyhow::{Context, Result};
use mlua::{Function, Table, Value};
use std::io::Write;

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
        self.register_diff(&sift)?;
        self.register_store(&sift)?;
        self.register_nudge(&sift)?;
        self.register_str(&sift)?;
        self.register_gain(&sift)?;
        self.register_meta(&sift)?;
        self.register_sift_ext(&sift)?;
        self.register_args(&sift)?;
        self.lua.globals().set("sift", sift)?;
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
        let plugin_table: Table = chunk
            .eval()
            .with_context(|| format!("failed to load plugin {name}"))?;

        let plugin_name: String = plugin_table.get("name")?;
        let priority: i32 = plugin_table.get("priority").unwrap_or(0);

        // Support pattern as string or string[]
        let patterns: Vec<String> = plugin_table
            .get::<String>("pattern")
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
            b_max.cmp(&a_max).then_with(|| b.priority.cmp(&a.priority))
        });

        // Rebuild pattern map after sorting so indices are correct
        self.pattern_map.clear();
        for (i, entry) in self.plugins.iter().enumerate() {
            for p in &entry.patterns {
                // Only insert if this plugin has higher priority than existing match
                let should_insert = self.pattern_map.get(p).map_or(true, |&existing_idx| {
                    let existing = &self.plugins[existing_idx];
                    let new_len = p.len();
                    let existing_max = existing.patterns.iter().map(String::len).max().unwrap_or(0);
                    entry.priority > existing.priority
                        || (entry.priority == existing.priority && new_len > existing_max)
                });
                if should_insert {
                    self.pattern_map.insert(p.clone(), i);
                }
            }
        }

        Ok(())
    }

    /// Find the best matching plugin for a command.
    fn find_plugin(&self, cmd: &str, args: &[String]) -> Option<&PluginEntry> {
        // Check exact cmd match first
        if let Some(&idx) = self.pattern_map.get(cmd) {
            return Some(&self.plugins[idx]);
        }

        // Check cmd + args combinations (longest first)
        let mut full = String::with_capacity(
            cmd.len() + args.iter().map(String::len).sum::<usize>() + args.len(),
        );
        full.push_str(cmd);
        for arg in args {
            full.push(' ');
            full.push_str(arg);
            if let Some(&idx) = self.pattern_map.get(&full) {
                return Some(&self.plugins[idx]);
            }
        }

        // Fall back to wildcard plugin
        self.pattern_map.get("*").map(|&idx| &self.plugins[idx])
    }

    /// Dispatch a full command string, handling cd/pushd/popd prefixes and pipelines.
    ///
    /// Returns `(output, exit_code, plugin_name)`.
    ///
    /// # Panics
    ///
    /// Panics if the Lua dispatch encounters an unexpected error.
    pub fn dispatch_full(
        &self,
        full_cmd: &str,
        stdin: Option<Value>,
    ) -> Result<(String, i32, String)> {
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

        // Try pipeline optimization
        if let Some(result) = self.try_pipeline(full_cmd)? {
            return Ok(result);
        }

        // Normal dispatch
        let parts = shlex::split(full_cmd)
            .unwrap_or_else(|| full_cmd.split_whitespace().map(String::from).collect());
        if parts.is_empty() {
            return Ok((String::new(), 0, String::new()));
        }
        let name = &parts[0];
        let args: Vec<String> = parts[1..].to_vec();

        // Handle file redirects
        self.dispatch_with_redirect(name, &args, stdin)
    }

    /// Try pipeline optimization, returning `Some(result)` if the last segment matches a plugin.
    fn try_pipeline(&self, full_cmd: &str) -> Result<Option<(String, i32, String)>> {
        let trimmed = full_cmd.trim();
        if !trimmed.contains('|') || trimmed.contains("||") {
            return Ok(None);
        }

        let segments: Vec<&str> = split_pipeline(trimmed);
        if segments.len() <= 1 {
            return Ok(None);
        }

        let last_segment = match segments.last() {
            Some(s) => s.trim(),
            None => return Ok(None),
        };
        let last_parts: Vec<String> = shlex::split(last_segment)
            .unwrap_or_else(|| last_segment.split_whitespace().map(String::from).collect());
        if last_parts.is_empty() {
            return Ok(None);
        }

        let last_name = &last_parts[0];
        let last_args: Vec<String> = last_parts[1..].to_vec();

        if self.find_plugin(last_name, &last_args).is_none() {
            return Ok(None);
        }

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
            return Ok(Some((
                format!("{stdout}{stderr}"),
                exit_code,
                "pipeline".to_string(),
            )));
        }

        // Dispatch last segment to plugin with preceding stdout as stdin
        let reader = StdinReader::from_string(stdout);
        let ud = self
            .lua
            .create_userdata(reader)
            .map_err(|e| mlua::Error::external(format!("create userdata: {e}")))?;
        let result = self.dispatch(last_name, &last_args, Some(Value::UserData(ud)), false)?;
        Ok(Some(result))
    }

    /// Dispatch with file redirect handling (`<`, `>`, `>>`).
    fn dispatch_with_redirect(
        &self,
        name: &str,
        args: &[String],
        stdin: Option<Value>,
    ) -> Result<(String, i32, String)> {
        // Handle fd redirects (2>&1, 1>&2, etc.) — strip from args, set merge_stderr
        let (clean_args, merge_stderr) = parse_fd_redirects(args);
        let args = &clean_args;

        // Handle < file redirect — open file, create StdinReader, pass as stdin
        if let Some(pos) = args.iter().position(|a| a == "<") {
            if pos + 1 < args.len() {
                let file_path = &args[pos + 1];
                match std::fs::File::open(file_path) {
                    Ok(file) => {
                        let reader = StdinReader::from_file(file);
                        let mut clean_args = args.clone();
                        clean_args.remove(pos);
                        clean_args.remove(pos);
                        let ud = self
                            .lua
                            .create_userdata(reader)
                            .map_err(|e| mlua::Error::external(format!("create userdata: {e}")))?;
                        return self.dispatch(
                            name,
                            &clean_args,
                            Some(Value::UserData(ud)),
                            merge_stderr,
                        );
                    }
                    Err(e) => {
                        return Ok((
                            format!("sift: cannot open '{file_path}': {e}"),
                            1,
                            String::new(),
                        ));
                    }
                }
            }
        }

        // Handle > file and >> file redirect — capture output, write to file
        if let Some(pos) = args.iter().position(|a| a == ">" || a == ">>") {
            if pos + 1 < args.len() {
                let file_path = &args[pos + 1];
                let append = args[pos] == ">>";
                let mut clean_args = args.clone();
                clean_args.remove(pos);
                clean_args.remove(pos);
                let (output, exit_code, plugin) =
                    self.dispatch(name, &clean_args, stdin, merge_stderr)?;
                if exit_code == 0 {
                    if append {
                        let _ = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(file_path)
                            .and_then(|mut f| f.write_all(output.as_bytes()));
                    } else {
                        let _ = std::fs::write(file_path, &output);
                    }
                }
                return Ok((output, exit_code, plugin));
            }
        }

        self.dispatch(name, args, stdin, merge_stderr)
    }

    /// Dispatch a command to the best matching plugin.
    ///
    /// Returns `(output, exit_code, plugin_name)`.
    pub fn dispatch(
        &self,
        cmd: &str,
        args: &[String],
        stdin: Option<Value>,
        merge_stderr: bool,
    ) -> Result<(String, i32, String)> {
        let entry = self.find_entry(cmd, args)?;

        let plugin_table: Table = self.lua.registry_value(&entry.table)?;
        let execute: Function = plugin_table.get("execute")?;

        // Build context table from pre-created template, update changing fields
        let ctx = match self.ctx_template_key.as_ref() {
            Some(key) => {
                let t: Table = self.lua.registry_value(key)?;
                t.set("cmd_count", self.ctx.cmd_count)?;
                t.set("command", cmd)?;
                t.set("merge_stderr", merge_stderr)?;
                t
            }
            None => {
                let t = self.lua.create_table()?;
                t.set("cwd", self.ctx.cwd_str.as_str())?;
                t.set("session_id", self.session_id_str.as_str())?;
                t.set("cmd_count", self.ctx.cmd_count)?;
                t.set("command", cmd)?;
                t.set("merge_stderr", merge_stderr)?;
                t
            }
        };

        // Build args table (arguments only, no command name)
        let args_table = self.lua.create_table()?;
        for (i, arg) in args.iter().enumerate() {
            args_table.set(i + 1, arg.as_str())?;
        }

        let stdin_val = stdin.unwrap_or(Value::Nil);

        // Clear nudges from previous dispatch
        if let Ok(mut guard) = self.nudges.lock() {
            guard.clear();
        }

        let result: Table = execute.call((ctx, args_table, stdin_val))?;

        let status: String = result.get("status")?;
        let output: String = result.get("output").unwrap_or_default();
        let exit_code: i32 = result.get("exit_code").unwrap_or(0);

        if status == "passthrough" {
            let (passthrough_output, passthrough_exit_code, _) =
                Self::execute_passthrough(cmd, args)?;
            let raw = i64::try_from(passthrough_output.len()).unwrap_or(i64::MAX);
            self.record_conversation(
                cmd,
                Some(raw),
                Some(raw),
                Some("command".to_string()),
                Some("passthrough".to_string()),
            );
            return Ok((
                passthrough_output,
                passthrough_exit_code,
                "command".to_string(),
            ));
        }

        let final_output = if status == "unchanged" {
            let mut msg = Self::handle_unchanged(&result);

            // Burst detection: track recent unchanged responses
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let key = format!("{cmd}:{msg}");
            if let Ok(mut recent) = self.recent_unchanged.lock() {
                // Prune entries older than 10 seconds
                recent.retain(|(_, ts)| now.saturating_sub(*ts) < 10_000);
                recent.push((key.clone(), now));
                // Keep sliding window of last 10
                while recent.len() > 10 {
                    recent.remove(0);
                }
                // Count occurrences of this key in the window
                let count = recent.iter().filter(|(k, _)| k == &key).count();
                if count >= 3 {
                    msg = format!(
                            "{msg}\n[sift] (this will keep returning the same result until the file changes on disk)",
                        );
                }
            }

            // Write unchanged message directly to stdout (for real-time visibility)
            print!("{msg}");
            let _ = std::io::stdout().flush();
            msg
        } else {
            // Write handled output directly to stdout (unless already streamed by plugin)
            let streamed: bool = result.get::<bool>("streamed").unwrap_or(false);
            if !streamed && !output.is_empty() {
                print!("{output}");
                let _ = std::io::stdout().flush();
            }
            output
        };

        let nudge_text = self.collect_nudges();
        // Write nudges directly to stdout (for real-time visibility)
        if !nudge_text.is_empty() {
            print!("{nudge_text}");
            let _ = std::io::stdout().flush();
        }
        let final_output = if nudge_text.is_empty() {
            final_output
        } else {
            format!("{final_output}{nudge_text}")
        };

        // Extract raw_bytes from plugin result (optional)
        let raw_bytes: Option<i64> = result.get::<Option<i64>>("raw_bytes").unwrap_or_default();
        let filtered_bytes = i64::try_from(final_output.len()).unwrap_or(i64::MAX);
        let plugin_name = entry.patterns.first().cloned();
        let output_format = if status == "unchanged" {
            Some("unchanged".to_string())
        } else {
            Some("text".to_string())
        };
        self.record_conversation(
            cmd,
            raw_bytes.or(Some(filtered_bytes)),
            Some(filtered_bytes),
            plugin_name,
            output_format,
        );

        Ok((
            final_output,
            exit_code,
            entry.patterns.first().cloned().unwrap_or_default(),
        ))
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

    /// Record a command execution into the `conversation_cache`.
    /// Spawns a background thread to avoid blocking dispatch.
    fn record_conversation(
        &self,
        _cmd: &str,
        raw_bytes: Option<i64>,
        filtered_bytes: Option<i64>,
        plugin_name: Option<String>,
        output_format: Option<String>,
    ) {
        let Some(store) = self.store.clone() else {
            return;
        };
        if self.session_id_str.is_empty() {
            return;
        }
        let item_id = format!("{}_{}", self.session_id_str, self.ctx.cmd_count);
        let cmd_count = i64::try_from(self.ctx.cmd_count).unwrap_or(i64::MAX);
        std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            rt.block_on(async move {
                let _ = store
                    .record_conversation(
                        "command_output",
                        &item_id,
                        None,
                        cmd_count,
                        raw_bytes,
                        filtered_bytes,
                        plugin_name,
                        output_format,
                    )
                    .await;
            });
        });
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

    /// Handle `status = "unchanged"` result: emit bypass nudge and return message.
    fn handle_unchanged(result: &Table) -> String {
        let msg: String = result.get("message").unwrap_or_default();
        msg
    }

    /// Execute a command directly (passthrough — bypass all plugins).
    fn execute_passthrough(cmd: &str, args: &[String]) -> Result<(String, i32, String)> {
        let full_cmd = if args.is_empty() {
            cmd.to_string()
        } else {
            format!("{} {}", cmd, args.join(" "))
        };
        // Use exec_command to execute the command directly
        let (stdout, stderr, exit_code) = exec_command(&full_cmd, "", 0, None, false, false)?;
        let combined = format!("{stdout}{stderr}");
        Ok((combined, exit_code, "passthrough".to_string()))
    }
}

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

/// Parse fd redirect patterns (e.g., `2>&1`, `1>&2`) from args.
/// Returns (`clean_args`, `merge_stderr`) where `merge_stderr` is true if `2>&1` was found.
/// Avoids allocation when no fd redirects are present.
fn parse_fd_redirects(args: &[String]) -> (Vec<String>, bool) {
    // Fast path: check if any arg is a fd redirect pattern
    let has_redirect = args.iter().any(|arg| {
        arg == "2>&1"
            || arg == "1>&2"
            || (arg.len() >= 4
                && arg
                    .as_bytes()
                    .iter()
                    .all(|b| b.is_ascii_digit() || *b == b'>' || *b == b'&')
                && arg.contains('>')
                && arg.contains('&'))
    });

    if !has_redirect {
        return (args.to_vec(), false);
    }

    let mut clean = Vec::with_capacity(args.len());
    let mut merge_stderr = false;
    for arg in args {
        if arg == "2>&1" {
            merge_stderr = true;
        } else if arg == "1>&2" {
            // Strip but no flag needed — bash handles naturally
        } else if arg.len() >= 4
            && arg
                .as_bytes()
                .iter()
                .all(|b| b.is_ascii_digit() || *b == b'>' || *b == b'&')
            && arg.contains('>')
            && arg.contains('&')
        {
            // Other N>&M patterns — strip but don't set flags for now
        } else {
            clean.push(arg.clone());
        }
    }
    (clean, merge_stderr)
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
pub fn compact_json(
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
                    .map(|v| {
                        compact_value(
                            v,
                            max_string_len,
                            max_array_items,
                            max_depth,
                            max_keys,
                            depth + 1,
                        )
                    })
                    .collect();
                let remaining = arr.len() - max_array_items;
                items.push(serde_json::Value::String(format!("... +{remaining} more")));
                serde_json::Value::Array(items)
            } else {
                serde_json::Value::Array(
                    arr.iter()
                        .map(|v| {
                            compact_value(
                                v,
                                max_string_len,
                                max_array_items,
                                max_depth,
                                max_keys,
                                depth + 1,
                            )
                        })
                        .collect(),
                )
            }
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<(String, serde_json::Value)> = obj
                .iter()
                .take(max_keys)
                .map(|(k, v)| {
                    (
                        k.clone(),
                        compact_value(
                            v,
                            max_string_len,
                            max_array_items,
                            max_depth,
                            max_keys,
                            depth + 1,
                        ),
                    )
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
