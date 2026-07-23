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
            .create_function(|_, (ctx, data): (Table, mlua::String)| {
                let _ = ctx; // ctx unused, accepted for API consistency
                Ok(hex::encode(sha2::Sha256::digest(data.as_bytes())))
            })?;
        hash.set("sha256", sha256_fn)?;
        let md5_fn = self
            .lua
            .create_function(|_, (ctx, data): (Table, mlua::String)| {
                let _ = ctx; // ctx unused, accepted for API consistency
                Ok(hex::encode(md5::Md5::digest(data.as_bytes())))
            })?;
        hash.set("md5", md5_fn)?;
        sift.set("hash", hash)?;
        Ok(())
    }

    pub(super) fn register_fs(&self, sift: &Table) -> Result<()> {
        let fs = self.lua.create_table()?;
        let fs_read = self.lua.create_function(
            |lua, (ctx, path, opts): (Table, String, Option<Table>)| {
                let _ = ctx;
                let offset: Option<usize> = opts.as_ref().and_then(|t| t.get("offset").ok());
                let limit: Option<usize> = opts.as_ref().and_then(|t| t.get("limit").ok());
                let bytes = std::fs::read(&path)
                    .map_err(|e| mlua::Error::external(format!("read {path}: {e}")))?;
                if offset.is_some() || limit.is_some() {
                    // Line-based slicing: find newline positions in the byte slice
                    let lines: Vec<&[u8]> = bytes.split(|b| *b == b'\n').collect();
                    let start = offset.unwrap_or(1).saturating_sub(1);
                    let end = limit.map_or(lines.len(), |l| start + l);
                    let selected: Vec<&[u8]> = lines
                        .iter()
                        .skip(start)
                        .take(end.saturating_sub(start))
                        .copied()
                        .collect();
                    let joined: Vec<u8> = selected.join(&b'\n');
                    Ok(lua
                        .create_string(&joined)
                        .map_err(|e| mlua::Error::external(format!("create string: {e}")))?)
                } else {
                    Ok(lua
                        .create_string(&bytes)
                        .map_err(|e| mlua::Error::external(format!("create string: {e}")))?)
                }
            },
        )?;
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
        let cmd_count = self.ctx.cmd_count.get();
        let nudges = self.nudges.clone();
        let shortest_fn = self.lua.create_function(
            move |_lua, (ctx, raw, formats): (Table, String, Table)| {
                let _ = ctx;
                Ok(json_shortest_impl(
                    &raw,
                    &formats,
                    &session_id,
                    cmd_count,
                    &nudges,
                ))
            },
        )?;
        json.set("shortest", shortest_fn)?;

        sift.set("json", json)?;

        let toon = self.lua.create_table()?;
        let toon_encode = self.lua.create_function(
            |lua, (_ctx, data, opts): (Table, Value, Option<Table>)| {
                let json_val = lua
                    .from_value::<serde_json::Value>(data)
                    .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))?;
                opts.map_or_else(
                    || {
                        toon_format::encode_default(&json_val)
                            .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))
                    },
                    |options| {
                        let mut encode_opts = toon_format::EncodeOptions::new();
                        if let Ok(delim) = options.get::<String>("delimiter") {
                            let d = match delim.as_str() {
                                "pipe" => toon_format::Delimiter::Pipe,
                                _ => toon_format::Delimiter::Comma,
                            };
                            encode_opts = encode_opts.with_delimiter(d);
                        }
                        if let Ok(indent) = options.get::<String>("indent") {
                            let i = match indent.as_str() {
                                "space4" => toon_format::Indent::Spaces(4),
                                _ => toon_format::Indent::Spaces(2),
                            };
                            encode_opts = encode_opts.with_indent(i);
                        }
                        toon_format::encode(&json_val, &encode_opts)
                            .map_err(|e| mlua::Error::external(format!("toon encode: {e}")))
                    },
                )
            },
        )?;
        toon.set("encode", toon_encode)?;
        let toon_decode =
            self.lua
                .create_function(|lua, (_ctx, s, opts): (Table, String, Option<Table>)| {
                    let result = match opts {
                        Some(options) => {
                            let strict = options.get::<bool>("strict").unwrap_or(false);
                            let no_coerce = options.get::<bool>("no_coerce").unwrap_or(false);
                            if strict && no_coerce {
                                return Err(mlua::Error::external(
                                    "toon decode: 'strict' and 'no_coerce' are mutually exclusive"
                                        .to_string(),
                                ));
                            }
                            if strict {
                                toon_format::decode_strict::<serde_json::Value>(&s)
                            } else if no_coerce {
                                toon_format::decode_no_coerce::<serde_json::Value>(&s)
                            } else {
                                toon_format::decode_default::<serde_json::Value>(&s)
                            }
                        }
                        None => toon_format::decode_default::<serde_json::Value>(&s),
                    };
                    let json_val =
                        result.map_err(|e| mlua::Error::external(format!("toon decode: {e}")))?;
                    lua.to_value(&json_val)
                        .map_err(|e| mlua::Error::external(format!("toon decode: {e}")))
                })?;
        toon.set("decode", toon_decode)?;
        sift.set("toon", toon)?;
        Ok(())
    }
}

/// Select the most token-efficient JSON representation.
fn json_shortest_impl(
    raw: &str,
    formats: &Table,
    session_id: &str,
    cmd_count: u64,
    nudges: &std::sync::Mutex<Vec<String>>,
) -> String {
    // Parse raw JSON — if invalid, return raw unchanged
    let json_val: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };

    // Always include compacted JSON as baseline (no pretty-print)
    let compact_raw = serde_json::to_string(&json_val).unwrap_or_else(|_| raw.to_string());
    let mut candidates: Vec<(String, String)> = Vec::new();
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
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(session_id);
    let nudge_path = tmp_dir.join(format!("{ts}_{cmd_count}_raw_original.json"));
    let nudge_path_str = nudge_path.display().to_string();
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
        let _ = std::fs::write(&nudge_path, raw);
        if let Ok(mut guard) = nudges.lock() {
            guard.push(format!("raw: 'command cat {nudge_path_str}'"));
        }
    }

    best_output.clone()
}

impl SiftLua {
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
        meta.set("cmd_count", ctx.cmd_count.get())?;
        meta.set("cwd", ctx.cwd.to_string_lossy().as_ref())?;
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
            .create_function(|lua, (_ctx, text): (Table, String)| {
                let tbl = lua.create_table()?;
                if text.is_empty() {
                    tbl.set(1, String::new())?;
                    return Ok(tbl);
                }
                for (i, line) in (1..).zip(text.lines()) {
                    tbl.set(i, line.to_string())?;
                }
                Ok(tbl)
            })?;
        str_tbl.set("split_lines", split_lines_fn)?;

        // sift.str.slice_text(text, start, end) -> string
        #[allow(clippy::cast_possible_truncation)]
        let slice_text_fn = self.lua.create_function(
            |_, (_ctx, text, start, end_): (Table, String, u64, u64)| {
                let lines: Vec<&str> = text.lines().collect();
                let total = lines.len();
                let s = (start.max(1) - 1) as usize;
                let e = (end_.min(total as u64)) as usize;
                if s >= total || s >= e {
                    return Ok(String::new());
                }
                let selected: Vec<&str> = lines.iter().skip(s).take(e - s).copied().collect();
                Ok(selected.join("\n"))
            },
        )?;
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
        let is_sensitive_fn =
            self.lua
                .create_function(move |_, (_ctx, path): (Table, String)| {
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

        // sift.str.shell_quote(str) -> string
        // Shell-quote a string for safe use in bash command construction.
        // Wraps in single quotes, escaping any embedded single quotes.
        let shell_quote_fn =
            self.lua
                .create_function(|_, (_ctx, s): (Table, String)| -> mlua::Result<String> {
                    Ok(crate::lua::api::sh_quote(&s))
                })?;
        str_tbl.set("shell_quote", shell_quote_fn)?;

        sift.set("str", str_tbl)?;
        Ok(())
    }

    pub(super) fn register_store(&self, sift: &Table) -> Result<()> {
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count.get();
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
                    guard.push(format!("raw: 'command cat {path_str}'"));
                }
                Ok(path_str)
            },
        )?;
        sift.set("store", store_fn)?;
        Ok(())
    }

    pub(super) fn register_gain(&self, sift: &Table) -> Result<()> {
        let store = self.store.clone();
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let gain_fn = self.lua.create_function(move |_, args: Table| {
            let store = match store {
                Some(ref s) => s.clone(),
                None => {
                    return Ok(
                        "No session store available. Set AI_SESSION to enable tracking."
                            .to_string(),
                    );
                }
            };
            let flags = parse_gain_flags(&args);
            let effective_session = flags.session.as_deref().or({
                if flags.all {
                    None
                } else {
                    Some(session_id.as_str())
                }
            });

            let report = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle
                    .block_on(generate_gain_report(&store, effective_session, &flags))
                    .map_err(|e| mlua::Error::external(format!("gain: query: {e}")))?
            } else if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(generate_gain_report(&store, effective_session, &flags))
                    .map_err(|e| mlua::Error::external(format!("gain: query: {e}")))?
            } else {
                return Ok("gain: failed to create tokio runtime".to_string());
            };

            if flags.json {
                serde_json::to_string(&report)
                    .map_err(|e| mlua::Error::external(format!("gain: json: {e}")))
            } else {
                Ok(format_gain_report(&report, effective_session))
            }
        })?;
        let gain = self.lua.create_table()?;
        gain.set("report", gain_fn)?;
        sift.set("gain", gain)?;
        Ok(())
    }
}

/// Flags for configuring the gain report output.
pub struct GainFlags {
    /// Show individual command list.
    pub verbose: bool,
    /// Output as JSON.
    pub json: bool,
    /// Show all sessions (not just current).
    pub all: bool,
    /// Filter by specific session ID.
    pub session: Option<String>,
    /// Filter by timestamp (unix ms).
    pub since: Option<i64>,
}

fn parse_gain_flags(args: &Table) -> GainFlags {
    GainFlags {
        verbose: args.get::<bool>("verbose").unwrap_or(false),
        json: args.get::<bool>("json").unwrap_or(false),
        all: args.get::<bool>("all").unwrap_or(false),
        session: args.get::<String>("session").ok(),
        since: args.get::<i64>("since").ok(),
    }
}

#[derive(serde::Serialize)]
/// Aggregated gain report data.
pub struct GainReport {
    total_commands: i64,
    total_raw_bytes: i64,
    total_filtered_bytes: i64,
    reduction_bps: i64,
    bypass_count: i64,
    session_count: Option<i64>,
    first_seen: Option<i64>,
    last_seen: Option<i64>,
    per_plugin: Vec<PluginGain>,
    commands: Option<Vec<CommandEntry>>,
}

#[derive(serde::Serialize)]
/// Per-plugin gain statistics.
pub struct PluginGain {
    plugin: String,
    calls: i64,
    raw_bytes: i64,
    filtered_bytes: i64,
    reduction_bps: i64,
}

#[derive(serde::Serialize)]
/// Individual command entry in verbose gain report.
pub struct CommandEntry {
    id: String,
    plugin: String,
    raw_bytes: i64,
    filtered_bytes: i64,
    reduction_bps: i64,
    output_format: String,
}

/// Query the session store and generate a gain report.
pub async fn generate_gain_report(
    store: &crate::session::SessionStore,
    session_id: Option<&str>,
    flags: &GainFlags,
) -> Result<GainReport, anyhow::Error> {
    let entries = store.query_conversations(session_id).await?;

    let mut total_commands: i64 = 0;
    let mut total_raw_bytes: i64 = 0;
    let mut total_filtered_bytes: i64 = 0;
    let mut bypass_count: i64 = 0;
    let mut plugin_map: std::collections::HashMap<String, (i64, i64, i64)> =
        std::collections::HashMap::new();
    let mut command_list = Vec::new();
    let mut session_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut first_seen: Option<i64> = None;
    let mut last_seen: Option<i64> = None;

    for entry in &entries {
        let plugin = entry
            .plugin_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let is_bypass = entry.output_format.as_deref() == Some("passthrough");
        let raw = entry.raw_bytes.unwrap_or(0);
        let filtered = entry.filtered_bytes.unwrap_or(0);

        // Track unique session IDs from item_id prefix (before first _)
        if let Some(underscore) = entry.item_id.find('_') {
            session_set.insert(entry.item_id[..underscore].to_string());
        }

        // Track date range
        if entry.first_shown > 0 {
            first_seen = Some(first_seen.map_or(entry.first_shown, |min| min.min(entry.first_shown)));
        }
        if entry.last_shown > 0 {
            last_seen = Some(last_seen.map_or(entry.last_shown, |max| max.max(entry.last_shown)));
        }

        total_commands += 1;
        total_raw_bytes += raw;
        total_filtered_bytes += filtered;

        if is_bypass {
            bypass_count += 1;
        }

        let p = plugin_map.entry(plugin.clone()).or_insert((0, 0, 0));
        p.0 += 1;
        p.1 += raw;
        p.2 += filtered;

        if flags.verbose {
            command_list.push(CommandEntry {
                id: entry.item_id.clone(),
                plugin,
                raw_bytes: raw,
                filtered_bytes: filtered,
                reduction_bps: entry.reduction_bps.unwrap_or(0),
                output_format: entry
                    .output_format
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            });
        }
    }

    let reduction_bps = if total_raw_bytes > 0 {
        (total_raw_bytes.saturating_sub(total_filtered_bytes)).saturating_mul(10_000)
            / total_raw_bytes
    } else {
        0
    };

    let mut per_plugin: Vec<PluginGain> = plugin_map
        .into_iter()
        .map(|(plugin, (calls, raw, filtered))| {
            let red = if raw > 0 {
                (raw.saturating_sub(filtered)).saturating_mul(10_000) / raw
            } else {
                0
            };
            PluginGain {
                plugin,
                calls,
                raw_bytes: raw,
                filtered_bytes: filtered,
                reduction_bps: red,
            }
        })
        .collect();
    per_plugin.sort_by_key(|a| std::cmp::Reverse(a.calls));

    Ok(GainReport {
        total_commands,
        total_raw_bytes,
        total_filtered_bytes,
        reduction_bps,
        bypass_count,
        session_count: if session_id.is_none() {
            Some(session_set.len() as i64)
        } else {
            None
        },
        first_seen,
        last_seen,
        per_plugin,
        commands: if flags.verbose {
            Some(command_list)
        } else {
            None
        },
    })
}

/// Convert a unix-ms timestamp to a YYYY-MM-DD date string.
fn timestamp_to_date(ts_ms: i64) -> Option<String> {
    let secs = ts_ms / 1000;
    let nanos = (ts_ms % 1000) as u32 * 1_000_000;
    let dt = chrono::DateTime::from_timestamp(secs, nanos)?;
    Some(dt.format("%Y-%m-%d").to_string())
}

/// Format a gain report as a human-readable string.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn format_gain_report(report: &GainReport, session_id: Option<&str>) -> String {
    use std::fmt::Write;
    let reduction_pct = (report.reduction_bps as f64) / 100.0;
    let mut out = String::new();
    let _ = writeln!(out, "sift gain");
    let _ = writeln!(out, "─────────────────────────────────────");
    // Commands line with optional session count
    if let Some(sc) = report.session_count {
        let _ = writeln!(out, "  Commands:    {}  (across {} sessions)", report.total_commands, sc);
    } else {
        let _ = writeln!(out, "  Commands:    {}", report.total_commands);
    }
    let _ = writeln!(out, "  Raw:         {} KB", report.total_raw_bytes / 1024);
    let _ = writeln!(
        out,
        "  Filtered:    {} KB",
        report.total_filtered_bytes / 1024
    );
    // Reduction line with absolute savings
    let saved_kb = (report.total_raw_bytes.saturating_sub(report.total_filtered_bytes)) / 1024;
    let _ = writeln!(
        out,
        "  Reduction:   {:.1}% ({} bps, {} KB saved)",
        reduction_pct, report.reduction_bps, saved_kb
    );
    let _ = writeln!(out, "  Bypasses:    {}", report.bypass_count);
    // Date range line
    if let (Some(first), Some(last)) = (report.first_seen, report.last_seen) {
        if let (Some(f_dt), Some(l_dt)) = (timestamp_to_date(first), timestamp_to_date(last)) {
            if f_dt == l_dt {
                let _ = writeln!(out, "  Period:      {}", f_dt);
            } else {
                let _ = writeln!(out, "  Period:      {} – {}", f_dt, l_dt);
            }
        }
    }
    let _ = writeln!(out, "  ─────────────────────────────────────");
    let _ = writeln!(out, "  Per plugin:");
    for p in &report.per_plugin {
        let pct = p.reduction_bps as f64 / 100.0;
        if p.plugin == "command" {
            let _ = writeln!(out, "    {:20} {:>4} calls   (bypass)", p.plugin, p.calls);
        } else {
            let _ = writeln!(
                out,
                "    {:20} {:>4} calls   {:.1}% reduction",
                p.plugin, p.calls, pct
            );
        }
    }

    if let Some(sid) = session_id {
        let _ = writeln!(out, "  ─────────────────────────────────────");
        let _ = writeln!(out, "  Session: {sid}");
    }

    if let Some(commands) = &report.commands {
        let _ = writeln!(out, "  ─────────────────────────────────────");
        let _ = writeln!(out, "  Commands:");
        for c in commands {
            let pct = c.reduction_bps as f64 / 100.0;
            let _ = writeln!(
                out,
                "    {:30} {:>8} → {:>8} ({:.1}%)",
                c.id, c.raw_bytes, c.filtered_bytes, pct
            );
        }
    }

    out
}
