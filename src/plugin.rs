//! Plugin trait, result types, and registry.

use anyhow::Result;
use async_trait::async_trait;

use crate::session::Session;

/// Result of a plugin execution.
pub enum PluginResult {
    /// Plugin handled the command. Output goes to stdout.
    Handled {
        /// Output bytes to write to stdout.
        output: Vec<u8>,
        /// Exit code for the command.
        exit_code: i32,
    },
    /// Plugin did not handle this command. Fall through to exec real binary.
    Passthrough,
    /// Output is identical to a previous invocation. Emit a short marker.
    Unchanged {
        /// Fingerprint for cache tracking.
        fingerprint: String,
        /// Human-readable message explaining the cache hit.
        message: String,
    },
}

/// A plugin that can intercept and handle a shell command.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Command name this plugin handles (e.g. "cat", "git").
    fn name(&self) -> &str;

    /// Execute the command.
    ///
    /// `args` is the command arguments (excluding the command name).
    /// `stdin` is the piped input, if any (None when reading from PTY).
    async fn execute(
        &self,
        session: &mut Session,
        args: &[String],
        stdin: Option<&[u8]>,
    ) -> Result<PluginResult>;
}

/// Registry mapping command names to plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Find a plugin by command name.
    pub fn find(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.name() == name).map(Box::as_ref)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    #[async_trait]
    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            "test-cmd"
        }

        async fn execute(
            &self,
            _session: &mut Session,
            _args: &[String],
            _stdin: Option<&[u8]>,
        ) -> Result<PluginResult> {
            Ok(PluginResult::Handled {
                output: b"test output".to_vec(),
                exit_code: 0,
            })
        }
    }

    #[test]
    fn test_register_and_find() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin));

        let found = registry.find("test-cmd");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "test-cmd");
    }

    #[test]
    fn test_find_unknown() {
        let registry = PluginRegistry::new();
        assert!(registry.find("nonexistent").is_none());
    }
}
