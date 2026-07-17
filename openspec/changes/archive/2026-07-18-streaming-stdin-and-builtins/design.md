## Context

sift's plugin dispatch currently passes stdin as `Option<&str>` — a pre-loaded string. For `< file` redirects, `dispatch_full` reads the entire file into memory via `std::fs::read_to_string()`. This causes OOM on large files. For piped input, the pipeline handler collects all preceding command output into a string before passing to the plugin.

String utility functions (`split_lines`, `slice_text`, `is_sensitive`) are defined identically in 4 plugin files (head.lua, tail.lua, sed.lua, sift-read.lua), plus `is_sensitive` duplicated in cat.lua. ~80 lines of duplicated Lua.

The pi extension (`integrations/pi/sift.ts`) swallows all errors in `resetCache` with `catch {}` and resets cache on 5 session events. Session ID is UUIDv7 — fork/switch create new IDs, naturally isolating the cache.

## Goals / Non-Goals

**Goals:**
- Replace `stdin: Option<&str>` with a streaming reader that plugins read incrementally
- Register `sift.str.split_lines()`, `sift.str.slice_text()`, `sift.str.is_sensitive()` as Rust-backed Lua builtins
- Remove duplicated Lua string utilities from all plugins
- Notify user on cache reset success/failure with hit detection
- Only reset cache on `session_compact`

**Non-Goals:**
- Full POSIX shell parser (out of scope — sift delegates complex commands to bash)
- Changing the plugin return API (only stdin input side changes)
- Performance optimization of existing `sift.fs.read()` (separate concern)

## Decisions

### 1. StdinReader Lua userdata instead of file path in context

**Decision**: Create a `StdinReader` Lua userdata type wrapping `Box<dyn Read + Send>`. The Rust layer opens the file (or wraps a `Cursor<String>` for piped input) and passes the reader to the plugin. The plugin reads incrementally via `stdin:readline()`, `stdin:read(n)`, or `stdin:lines()`.

**Alternatives considered**:
- *File path in context*: Leaks file I/O into plugin logic, requires plugins to handle paths
- *Read entire file in Rust*: Current approach — OOM on large files
- *Delegate to bash*: Breaks plugin interception for commands with `< file`

**Rationale**: Plugin reads stdin the same way regardless of source (pipe, file redirect, string). Rust handles all I/O. No OOM. No file path leaking.

### 2. `sift.str.*` as Rust functions instead of Lua libraries

**Decision**: Implement `split_lines`, `slice_text`, `is_sensitive` as Rust functions registered in the Lua VM, alongside existing `sift.fs.*`, `sift.json.*`, etc.

**Alternatives considered**:
- *Lua module file*: Would need file loading infrastructure, slower than Rust
- *Keep duplicated Lua code*: Current approach — maintenance burden

**Rationale**: Rust string processing is faster than Lua `gmatch`. Registration follows existing pattern (`api_reg_io.rs`). Zero additional dependencies.

### 3. Only reset cache on `session_compact`

**Decision**: Remove `session_shutdown`, `session_tree`, `session_fork`, `session_switch` handlers. Keep only `session_compact`.

**Rationale**: Session ID is UUIDv7 — fork/switch create new IDs, naturally isolating `/tmp/sift/<session>/` directories. `session_shutdown` may be followed by `pi -c` restarting the same session. `session_tree` stays in the same session. `session_compact` is the only event where conversation context changes while staying in the same session.

## Risks / Trade-offs

- **[API breakage]** Existing plugins that read `stdin` as a string will break. They must check `type(stdin)` and use the reader API. → Mitigation: `StdinReader` also supports `tostring()` for backward compat with small inputs.
- **[Performance]** Wrapping `BufReader<File>` in Lua userdata adds overhead per read call vs. a single `read_to_string`. → Mitigation: For small inputs (< 64KB), use `Cursor<String>` which is essentially free.
- **[Complexity]** `StdinReader` userdata requires mlua `UserData` impl with methods. → Mitigation: Follow existing mlua patterns (see `sift.fs` registration for reference).
