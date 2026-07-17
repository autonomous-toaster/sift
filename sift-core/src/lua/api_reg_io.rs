use super::api::compact_json;
use super::SiftLua;
use crate::classifier::classify;

use anyhow::Result;
use jaq_interpret::{FilterT, RcIter};
use mlua::{Lua, LuaSerdeExt, Table, Value};
use sha2::Digest;

use serde_json;

impl SiftLua {
    pub(super) fn register_hash(&self, sift: &Table) -> Result<()> {
        let hash = self.lua.create_table()?;
        let sha256_fn = self
            .lua
            .create_function(|_, (ctx, data): (Table, String)| {
                let _ = ctx; // ctx unused, accepted for API consistency
                Ok(hex::encode(sha2::Sha256::digest(data.as_bytes())))
            })?;
        hash.set("sha256", sha256_fn)?;
        let md5_fn = self
            .lua
            .create_function(|_, (ctx, data): (Table, String)| {
                let _ = ctx; // ctx unused, accepted for API consistency
                Ok(hex::encode(md5::Md5::digest(data.as_bytes())))
            })?;
        hash.set("md5", md5_fn)?;
        sift.set("hash", hash)?;
        Ok(())
    }

    pub(super) fn register_fs(&self, sift: &Table) -> Result<()> {
        let fs = self.lua.create_table()?;
        let fs_read =
            self.lua
                .create_function(|_, (ctx, path, opts): (Table, String, Option<Table>)| {
                    let _ = ctx;
                    let offset: Option<usize> = opts.as_ref().and_then(|t| t.get("offset").ok());
                    let limit: Option<usize> = opts.as_ref().and_then(|t| t.get("limit").ok());
                    let content = std::fs::read_to_string(&path)
                        .map_err(|e| mlua::Error::external(format!("read {path}: {e}")))?;
                    let lines: Vec<&str> = content.lines().collect();
                    let start = offset.unwrap_or(1).saturating_sub(1);
                    let end = limit.map_or(lines.len(), |l| start + l);
                    let selected: Vec<&str> = lines
                        .iter()
                        .skip(start)
                        .take(end.saturating_sub(start))
                        .copied()
                        .collect();
                    Ok(selected.join("\n"))
                })?;
        fs.set("read", fs_read)?;

        // fs.write(path, content)
        let fs_write =
            self.lua
                .create_function(|_, (ctx, path, content): (Table, String, String)| {
                    let _ = ctx;
                    std::fs::write(&path, &content)
                        .map_err(|e| mlua::Error::external(format!("write {path}: {e}")))?;
                    Ok(())
                })?;
        fs.set("write", fs_write)?;

        // fs.edit(path, edits) — apply multiple disjoint text replacements
        let fs_edit =
            self.lua
                .create_function(|_, (ctx, path, edits): (Table, String, Table)| {
                    let _ = ctx;
                    let mut content = std::fs::read_to_string(&path)
                        .map_err(|e| mlua::Error::external(format!("read {path}: {e}")))?;
                    let num_edits = usize::try_from(
                        edits
                            .len()
                            .map_err(|e| mlua::Error::external(e.to_string()))?,
                    )
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
        let fs_stat = self
            .lua
            .create_function(|lua, (ctx, path): (Table, String)| {
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
        let fs_exists = self
            .lua
            .create_function(|_, (ctx, path): (Table, String)| {
                let _ = ctx;
                Ok(std::path::Path::new(&path).exists())
            })?;
        fs.set("exists", fs_exists)?;

        sift.set("fs", fs)?;
        Ok(())
    }

    pub(super) fn register_json_toon(&self, sift: &Table) -> Result<()> {
        let json = self.lua.create_table()?;
        let json_encode = self
            .lua
            .create_function(|lua, (ctx, val): (Table, Value)| {
                let _ = ctx;
                let json_val = lua
                    .from_value::<serde_json::Value>(val)
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
        let shortest_fn = self.lua.create_function(
            move |_lua, (ctx, raw, formats): (Table, String, Table)| {
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
                    let compacted = compact_json(
                        &json_val,
                        max_string_len,
                        max_array_items,
                        max_depth,
                        max_keys,
                    );
                    candidates.push(("json".to_string(), compacted));
                }

                // Check if toon format is requested
                let has_toon = formats.get::<bool>("toon").unwrap_or(false)
                    || formats
                        .get::<Option<Table>>("toon")
                        .ok()
                        .flatten()
                        .is_some();
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
            },
        )?;
        json.set("shortest", shortest_fn)?;

        sift.set("json", json)?;

        let toon = self.lua.create_table()?;
        let toon_encode = self
            .lua
            .create_function(|lua, (ctx, val): (Table, Value)| {
                let _ = ctx;
                let json_val = lua
                    .from_value::<serde_json::Value>(val)
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

    pub(super) fn register_jq(&self, sift: &Table) -> Result<()> {
        let jq = self.lua.create_table()?;
        let jq_query =
            self.lua
                .create_function(|lua, (ctx, data, filter): (Table, Value, String)| {
                    let _ = ctx;
                    let json_val: serde_json::Value = if let Value::String(s) = &data {
                        let s = s
                            .to_str()
                            .map_err(|e| mlua::Error::external(format!("jq str: {e}")))?;
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
                    let cv = (
                        jaq_interpret::Ctx::new([], &inputs),
                        jaq_interpret::Val::from(json_val),
                    );
                    let mut outputs = Vec::new();
                    for val in filter.run(cv) {
                        match val {
                            Ok(v) => {
                                let s = format!("{v}");
                                if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&s) {
                                    outputs.push(jv);
                                }
                            }
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

    pub(super) fn register_env(&self, sift: &Table) -> Result<()> {
        let env = self.lua.create_table()?;
        let env_get = self.lua.create_function(|_, (ctx, key): (Table, String)| {
            let _ = ctx;
            Ok(std::env::var(key).ok())
        })?;
        env.set("get", env_get)?;
        let env_set = self
            .lua
            .create_function(|_, (ctx, key, val): (Table, String, String)| {
                let _ = ctx;
                std::env::set_var(&key, &val);
                Ok(())
            })?;
        env.set("set", env_set)?;
        sift.set("env", env)?;
        Ok(())
    }

    pub(super) fn register_classify(&self, sift: &Table) -> Result<()> {
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

        let token_count_fn = self
            .lua
            .create_function(|_, (_ctx, text): (Table, String)| {
                Ok(i64::try_from(text.len() / 4).unwrap_or(i64::MAX))
            })?;
        sift.set("token_count", token_count_fn)?;
        Ok(())
    }

    pub(super) fn register_diff(&self, sift: &Table) -> Result<()> {
        let diff_fn =
            self.lua
                .create_function(|_, (_ctx, old, new): (Table, String, String)| {
                    let diff = similar::TextDiff::from_lines(&old, &new)
                        .unified_diff()
                        .context_radius(3)
                        .to_string();
                    Ok(diff)
                })?;
        sift.set("diff", diff_fn)?;
        Ok(())
    }

    pub(super) fn register_meta(&self, sift: &Table) -> Result<()> {
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

    pub(super) fn register_str(&self, sift: &Table) -> Result<()> {
        let str_tbl = self.lua.create_table()?;

        // sift.str.split_lines(text) -> { line1, line2, ... }
        let split_lines_fn = self
            .lua
            .create_function(|lua, text: String| {
                let tbl = lua.create_table()?;
                if text.is_empty() {
                    tbl.set(1, String::new())?;
                    return Ok(tbl);
                }
                let mut i = 1;
                for line in text.lines() {
                    tbl.set(i, line.to_string())?;
                    i += 1;
                }
                if text.ends_with('\n') {
                    tbl.set(i, String::new())?;
                }
                Ok(tbl)
            })?;
        str_tbl.set("split_lines", split_lines_fn)?;

        // sift.str.slice_text(text, start, end) -> string
        #[allow(clippy::cast_possible_truncation)]
        let slice_text_fn = self
            .lua
            .create_function(|_, (text, start, end_): (String, u64, u64)| {
                let lines: Vec<&str> = text.lines().collect();
                let total = if text.ends_with('\n') {
                    lines.len() + 1
                } else {
                    lines.len()
                };
                let s = (start.max(1) - 1) as usize;
                let e = (end_.min(total as u64)) as usize;
                if s >= total || s >= e {
                    return Ok(String::new());
                }
                let selected: Vec<&str> = lines.iter().skip(s).take(e - s).copied().collect();
                Ok(selected.join("\n"))
            })?;
        str_tbl.set("slice_text", slice_text_fn)?;

        // sift.str.is_sensitive(path) -> bool
        let sensitive_patterns: &[(&str, bool, bool)] = &[
            (".env", true, false),
            (".pem", false, true),
            (".key", false, true),
            (".p12", false, true),
            (".pfx", false, true),
            (".crt", false, true),
            (".cer", false, true),
            (".der", false, true),
            (".pk8", false, true),
            ("id_rsa", false, false),
            ("id_ed25519", false, false),
            (".npmrc", false, true),
            (".netrc", false, true),
        ];
        let is_sensitive_fn = self
            .lua
            .create_function(move |_, path: String| {
                let lower = path.to_lowercase();
                Ok(sensitive_patterns.iter().any(|(pat, prefix, suffix)| {
                    if *prefix {
                        lower.starts_with(pat)
                    } else if *suffix {
                        lower.ends_with(pat)
                    } else {
                        lower.contains(pat)
                    }
                }))
            })?;
        str_tbl.set("is_sensitive", is_sensitive_fn)?;

        sift.set("str", str_tbl)?;
        Ok(())
    }

    pub(super) fn register_store(&self, sift: &Table) -> Result<()> {
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count;
        let nudges = self.nudges.clone();
        let store_fn = self.lua.create_function(
            move |_, (_ctx, content, slug): (Table, String, String)| {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(&session_id);
                let _ = std::fs::create_dir_all(&tmp_dir);
                let safe_slug: String = slug
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect();
                let path = tmp_dir.join(format!("{ts}_{cmd_count}_{safe_slug}"));
                let path_str = path.display().to_string();
                let _ = std::fs::write(&path, &content);
                if let Ok(mut guard) = nudges.lock() {
                    guard.push(format!("stored: 'command cat {path_str}'"));
                }
                Ok(path_str)
            },
        )?;
        sift.set("store", store_fn)?;
        Ok(())
    }
}
