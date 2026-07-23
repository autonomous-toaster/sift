use super::SiftLua;
use crate::lua::exec::{exec_command, save_output, TransformFn};
use anyhow::Result;
use mlua::Table;
use sha2::Digest;

use serde_json;

impl SiftLua {
    pub(super) fn register_exec(&self, sift: &Table) -> Result<()> {
        let session_id = self.ctx.session_id.clone().unwrap_or_default();
        let cmd_count = self.ctx.cmd_count.get();
        let nudges = self.nudges.clone();
        let lua = self.lua.clone();
        let exec_fn = self.lua.create_function(
            move |_, (_ctx, cmd, opts): (Table, String, Option<Table>)| {
                // Extract optional transform function and silent flag from opts
                let transform: Option<TransformFn> = opts
                    .as_ref()
                    .and_then(|t| t.get::<mlua::Function>("transform").ok())
                    .and_then(|func| {
                        let lua_clone = lua.clone();
                        let Ok(key) = lua.create_registry_value(func) else {
                            return None;
                        };
                        let b: TransformFn = Box::new(move |chunk: &str| -> String {
                            lua_clone
                                .registry_value::<mlua::Function>(&key)
                                .ok()
                                .and_then(|f| f.call::<String>(chunk).ok())
                                .unwrap_or_else(|| chunk.to_string())
                        });
                        Some(b)
                    });
                let silent = opts
                    .as_ref()
                    .and_then(|t| t.get::<bool>("silent").ok())
                    .unwrap_or(false);
                let merge_stderr = opts
                    .as_ref()
                    .and_then(|t| t.get::<bool>("merge_stderr").ok())
                    .unwrap_or(false);

                let (stdout, stderr, exit_code) = exec_command(
                    &cmd,
                    &session_id,
                    cmd_count,
                    transform,
                    silent,
                    merge_stderr,
                    None,
                )?;
                let combined = format!("{stdout}{stderr}");
                // On-error save with auto-nudge
                if exit_code != 0 {
                    let path = save_output(&cmd, &session_id, cmd_count, &combined);
                    if let Ok(mut guard) = nudges.lock() {
                        guard.push(format!("raw: 'command cat {path}'"));
                    }
                }
                Ok((combined, stderr, exit_code))
            },
        )?;
        sift.set("exec", exec_fn)?;
        self.register_log(sift)?;

        let exit_fn =
            self.lua
                .create_function(|_, (_ctx, code): (Table, i32)| -> mlua::Result<()> {
                    std::process::exit(code);
                })?;
        sift.set("exit", exit_fn)?;

        let output_fn = self
            .lua
            .create_function(|_, (_ctx, text): (Table, String)| {
                print!("{text}");
                Ok(())
            })?;
        sift.set("output", output_fn)?;
        Ok(())
    }

    pub(super) fn register_log(&self, sift: &Table) -> Result<()> {
        let log_table = self.lua.create_table()?;

        let info_fn = self
            .lua
            .create_function(|_, (_ctx, msg): (Table, String)| {
                println!("[sift] INFO: {msg}");
                Ok(())
            })?;
        log_table.set("info", info_fn)?;

        let warn_fn = self
            .lua
            .create_function(|_, (_ctx, msg): (Table, String)| {
                eprintln!("[sift] WARN: {msg}");
                Ok(())
            })?;
        log_table.set("warn", warn_fn)?;

        let error_fn = self
            .lua
            .create_function(|_, (_ctx, msg): (Table, String)| {
                eprintln!("[sift] ERROR: {msg}");
                Ok(())
            })?;
        log_table.set("error", error_fn)?;

        let debug_fn = self
            .lua
            .create_function(|_, (_ctx, msg): (Table, String)| {
                println!("[sift] DEBUG: {msg}");
                Ok(())
            })?;
        log_table.set("debug", debug_fn)?;

        sift.set("log", log_table)?;
        Ok(())
    }

    pub(super) fn register_nudge(&self, sift: &Table) -> Result<()> {
        let nudges = self.nudges.clone();
        let nudge_fn = self
            .lua
            .create_function(move |_, (_ctx, msg): (Table, String)| {
                if let Ok(mut guard) = nudges.lock() {
                    guard.push(msg);
                }
                Ok(())
            })?;
        sift.set("nudge", nudge_fn)?;
        Ok(())
    }

    pub(super) fn register_cache(&self, sift: &Table) -> Result<()> {
        let cache = self.lua.create_table()?;
        self.register_cache_in_memory(&cache)?;
        self.register_cache_file_ops(&cache)?;
        sift.set("cache", cache)?;
        Ok(())
    }

    /// Register in-memory cache operations: has, set, reset (per-invocation).
    pub(super) fn register_cache_in_memory(&self, cache: &Table) -> Result<()> {
        let store: Option<std::sync::Arc<crate::session::SessionStore>> = self.store.clone();

        let f_has = self
            .lua
            .create_function(move |_, (ctx, key): (Table, String)| {
                let session_id: String = ctx.get("session_id")?;
                store.as_ref().map_or_else(
                    || Ok(false),
                    |s| match futures::executor::block_on(s.cache_has(&key, &session_id)) {
                        Ok(v) => Ok(v),
                        Err(e) => Err(mlua::Error::external(e.to_string())),
                    },
                )
            })?;
        cache.set("has", f_has)?;

        let store2 = self.store.clone();
        let f_set = self
            .lua
            .create_function(move |_, (ctx, key): (Table, String)| {
                let session_id: String = ctx.get("session_id")?;
                if let Some(ref store) = store2 {
                    futures::executor::block_on(store.cache_set(&key, &session_id))
                        .map_err(|e| mlua::Error::external(e.to_string()))?;
                }
                Ok(())
            })?;
        cache.set("set", f_set)?;

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
        Ok(())
    }

    /// Register file-based cache operations: `has_file`, `store_file`, `load_file`, `cleanup`, `clear_all`.
    /// These persist across invocations within the same `AI_SESSION`.
    pub(super) fn register_cache_file_ops(&self, cache: &Table) -> Result<()> {
        self.register_cache_file_has(cache)?;
        self.register_cache_file_store(cache)?;
        self.register_cache_file_store_content(cache)?;
        self.register_cache_file_load(cache)?;
        self.register_cache_path_hash(cache)?;
        self.register_cache_range_ops(cache)?;
        self.register_cache_cleanup(cache)?;
        self.register_cache_clear_all(cache)?;
        Ok(())
    }

    pub(super) fn register_cache_file_has(&self, cache: &Table) -> Result<()> {
        let f = self
            .lua
            .create_function(|_, (ctx, hash): (Table, String)| {
                let session_id: String = ctx.get("session_id")?;
                let marker_path = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("cache")
                    .join(&hash);
                // Only return true if marker exists AND has "full": true
                // (range-only markers from add_range should not satisfy has_file)
                std::fs::read_to_string(&marker_path).map_or_else(
                    |_| Ok(false),
                    |s| {
                        serde_json::from_str::<serde_json::Value>(&s)
                            .map(|meta| {
                                meta.get("full")
                                    .and_then(serde_json::Value::as_bool)
                                    .unwrap_or(false)
                            })
                            .map_err(|e| mlua::Error::external(format!("has_file parse: {e}")))
                    },
                )
            })?;
        cache.set("has_file", f)?;
        Ok(())
    }

    pub(super) fn register_cache_file_store(&self, cache: &Table) -> Result<()> {
        let f = self
            .lua
            .create_function(|_, (ctx, hash, content): (Table, String, String)| {
                let session_id: String = ctx.get("session_id")?;
                let base = std::path::PathBuf::from("/tmp/sift").join(&session_id);

                let objects_dir = base.join("objects");
                std::fs::create_dir_all(&objects_dir)
                    .map_err(|e| mlua::Error::external(format!("store objects dir: {e}")))?;
                let object_path = objects_dir.join(format!("sha256-{hash}.txt"));
                std::fs::write(&object_path, &content)
                    .map_err(|e| mlua::Error::external(format!("store object: {e}")))?;

                let cache_dir = base.join("cache");
                std::fs::create_dir_all(&cache_dir)
                    .map_err(|e| mlua::Error::external(format!("store cache dir: {e}")))?;
                let marker_path = cache_dir.join(&hash);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                let meta = serde_json::json!({
                    "created_at": now,
                    "size": content.len(),
                    "full": true
                });
                std::fs::write(&marker_path, meta.to_string())
                    .map_err(|e| mlua::Error::external(format!("store marker: {e}")))?;

                Ok(())
            })?;
        cache.set("store_file", f)?;
        Ok(())
    }

    pub(super) fn register_cache_file_store_content(&self, cache: &Table) -> Result<()> {
        let f = self
            .lua
            .create_function(|_, (ctx, hash, content): (Table, String, String)| {
                let session_id: String = ctx.get("session_id")?;
                let objects_dir = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("objects");
                std::fs::create_dir_all(&objects_dir)
                    .map_err(|e| mlua::Error::external(format!("store objects dir: {e}")))?;
                let object_path = objects_dir.join(format!("sha256-{hash}.txt"));
                std::fs::write(&object_path, &content)
                    .map_err(|e| mlua::Error::external(format!("store object: {e}")))?;
                Ok(())
            })?;
        cache.set("store_content", f)?;
        Ok(())
    }

    pub(super) fn register_cache_file_load(&self, cache: &Table) -> Result<()> {
        let f = self
            .lua
            .create_function(|_, (ctx, hash): (Table, String)| {
                let session_id: String = ctx.get("session_id")?;
                let object_path = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("objects")
                    .join(format!("sha256-{hash}.txt"));
                std::fs::read_to_string(&object_path)
                    .map_or_else(|_| Ok(None), |content| Ok(Some(content)))
            })?;
        cache.set("load_file", f)?;
        Ok(())
    }

    pub(super) fn register_cache_path_hash(&self, cache: &Table) -> Result<()> {
        // sift.cache.set_path_hash(ctx, path, hash)
        let f_set = self
            .lua
            .create_function(|_, (ctx, path, hash): (Table, String, String)| {
                let session_id: String = ctx.get("session_id")?;
                let path_hash = hex::encode(sha2::Sha256::digest(path.as_bytes()));
                let marker_path = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("paths")
                    .join(&path_hash);
                if let Some(parent) = marker_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(&marker_path, &hash)
                    .map_err(|e| mlua::Error::external(format!("set path hash: {e}")))?;
                Ok(())
            })?;
        cache.set("set_path_hash", f_set)?;

        // sift.cache.get_path_hash(ctx, path) -> string|nil
        let f_get = self
            .lua
            .create_function(|_, (ctx, path): (Table, String)| {
                let session_id: String = ctx.get("session_id")?;
                let path_hash = hex::encode(sha2::Sha256::digest(path.as_bytes()));
                let marker_path = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("paths")
                    .join(&path_hash);
                std::fs::read_to_string(&marker_path)
                    .map_or_else(|_| Ok(None), |h| Ok(Some(h.trim().to_string())))
            })?;
        cache.set("get_path_hash", f_get)?;
        Ok(())
    }

    pub(super) fn register_cache_range_ops(&self, cache: &Table) -> Result<()> {
        // sift.cache.add_range(ctx, hash, start, end)
        let f_add = self.lua.create_function(
            |_, (ctx, hash, start, end_): (Table, String, u64, u64)| {
                let session_id: String = ctx.get("session_id")?;
                let marker_path = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("cache")
                    .join(&hash);

                // Load existing marker or create default
                let mut marker: serde_json::Value = std::fs::read_to_string(&marker_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({"ranges": []}));

                // Get or init ranges array
                let mut ranges: Vec<[u64; 2]> = marker["ranges"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                let pair = v.as_array()?;
                                let s = pair[0].as_u64()?;
                                let e = pair[1].as_u64()?;
                                Some([s, e])
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Add new range and merge
                ranges.push([start, end_]);
                ranges.sort_by_key(|r| r[0]);
                let mut merged: Vec<[u64; 2]> = Vec::new();
                for r in &ranges {
                    if let Some(last) = merged.last_mut() {
                        if r[0] <= last[1] + 1 {
                            last[1] = last[1].max(r[1]);
                            continue;
                        }
                    }
                    merged.push(*r);
                }

                // Update marker
                let ranges_json: Vec<Vec<u64>> = merged.iter().map(|r| vec![r[0], r[1]]).collect();
                marker["ranges"] = serde_json::json!(ranges_json);
                if marker.get("created_at").is_none() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    marker["created_at"] = serde_json::json!(now);
                }

                if let Some(parent) = marker_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(&marker_path, marker.to_string())
                    .map_err(|e| mlua::Error::external(format!("add range: {e}")))?;

                Ok(())
            },
        )?;
        cache.set("add_range", f_add)?;

        // sift.cache.has_range(ctx, hash, start, end) -> bool
        let f_has = self.lua.create_function(
            |_, (ctx, hash, start, end_): (Table, String, u64, u64)| {
                let session_id: String = ctx.get("session_id")?;
                let marker_path = std::path::PathBuf::from("/tmp/sift")
                    .join(&session_id)
                    .join("cache")
                    .join(&hash);

                let Ok(marker_str) = std::fs::read_to_string(&marker_path) else {
                    return Ok(false);
                };
                let marker: serde_json::Value = match serde_json::from_str(&marker_str) {
                    Ok(v) => v,
                    Err(_) => return Ok(false),
                };

                let contained = marker["ranges"].as_array().is_some_and(|arr| {
                    arr.iter().any(|v| {
                        v.as_array()
                            .and_then(|pair| {
                                let s = pair[0].as_u64()?;
                                let e = pair[1].as_u64()?;
                                Some(s <= start && e >= end_)
                            })
                            .unwrap_or(false)
                    })
                });

                Ok(contained)
            },
        )?;
        cache.set("has_range", f_has)?;
        Ok(())
    }

    pub(super) fn register_cache_cleanup(&self, cache: &Table) -> Result<()> {
        let f = self
            .lua
            .create_function(|_, (ctx, max_age_ms): (Table, Option<u64>)| {
                let session_id: String = ctx.get("session_id")?;
                let base = std::path::PathBuf::from("/tmp/sift").join(&session_id);
                let max_age = u128::from(max_age_ms.unwrap_or(86_400_000));
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
                                if now.saturating_sub(u128::from(created)) > max_age {
                                    let _ = std::fs::remove_file(entry.path());
                                    let obj_path = base
                                        .join("objects")
                                        .join(format!("sha256-{fname_str}.txt"));
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

                Ok(())
            })?;
        cache.set("cleanup", f)?;
        Ok(())
    }

    pub(super) fn register_cache_clear_all(&self, cache: &Table) -> Result<()> {
        // sift.cache.has_any(ctx) -> bool — check if cache has any entries
        let f_has_any = self.lua.create_function(|_, ctx: Table| {
            let session_id: String = ctx.get("session_id")?;
            let cache_dir = std::path::PathBuf::from("/tmp/sift")
                .join(&session_id)
                .join("cache");
            std::fs::read_dir(&cache_dir).map_or_else(
                |_| Ok(false),
                |entries| Ok(entries.flatten().next().is_some()),
            )
        })?;
        cache.set("has_any", f_has_any)?;

        let f = self.lua.create_function(|_, ctx: Table| {
            let session_id: String = ctx.get("session_id")?;
            let base = std::path::PathBuf::from("/tmp/sift").join(&session_id);

            let cache_dir = base.join("cache");
            if let Ok(entries) = std::fs::read_dir(&cache_dir) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            let objects_dir = base.join("objects");
            if let Ok(entries) = std::fs::read_dir(&objects_dir) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            Ok(())
        })?;
        cache.set("clear_all", f)?;
        Ok(())
    }
}
