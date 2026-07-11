# STD-003 · Plugin Architecture

## Plugin trait

Every plugin implements:

```rust
pub enum PluginResult {
    /// Plugin handled the command. Output goes to stdout.
    Handled { output: Vec<u8>, exit_code: i32 },
    /// Plugin did not handle this command. Fall through to exec real binary.
    Passthrough,
    /// Output is identical to a previous invocation. Emit a short marker.
    Unchanged { fingerprint: String, message: String },
}

pub trait Plugin: Send + Sync {
    /// Command name this plugin handles (e.g. "cat", "git").
    fn name(&self) -> &str;

    /// Execute the command.
    /// `args` is the command arguments (excluding the command name).
    /// `stdin` is the piped input, if any.
    fn execute(
        &self,
        session: &mut Session,
        args: &[String],
        stdin: Option<&[u8]>,
    ) -> Result<PluginResult>;
}
```

## Registry

The registry maps command names to plugins using longest-prefix matching:

```rust
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: Box<dyn Plugin>);
    pub fn find(&self, cmd: &str) -> Option<&dyn Plugin>;
}
```

Subcommand matching (e.g. `cargo build` vs `cargo test`) is handled by the plugin itself. The registry matches on the top-level command name only. The plugin receives the full args and dispatches internally.

## Interception rules

A plugin is only invoked when:
1. The command's stdout goes directly to the PTY/harness (not piped to another command, not redirected to a file).
2. The command name matches a registered plugin.
3. The plugin returns `Handled` or `Unchanged`.

If any condition is false, the command falls through to exec the real binary.

## Plugin responsibilities

Each plugin must:
- Accept all standard flags for the command it wraps (or return `Passthrough` for unsupported flags).
- Produce byte-for-byte identical output to the real command for the same input.
- Never silently drop or alter data — only compress whitespace, strip ANSI, or apply lossless transformations.
- Check the session store for cached results before executing.
- Update the session store after executing.
