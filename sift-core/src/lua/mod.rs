//! Lua runtime and `sift.*` API for plugins.
//!
//! Provides the mlua-based Lua VM, registers all `sift.*` functions,
//! and handles plugin loading and dispatch.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use mlua::Lua;

use crate::session::SessionStore;

pub(crate) mod api;
pub(crate) mod api_reg_cache;
/// API registration functions for I/O operations (hash, fs, json, toon, jq, env, classify, diff, meta, str, store, gain).
pub mod api_reg_io;
pub mod exec;
pub(crate) mod stdin_reader;
pub use exec::cleanup_cache;

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
    /// Recent unchanged responses for burst detection: (key, `timestamp_ms`).
    recent_unchanged: Arc<Mutex<Vec<(String, u128)>>>,
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
            recent_unchanged: Arc::new(Mutex::new(Vec::new())),
        };

        runtime.register_sift_table()?;
        Ok(runtime)
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_plugins;
