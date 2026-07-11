## Why

AI coding agents (Claude Code, Codex, Cursor, pi, etc.) consume tokens for every shell command output they process. A typical agent session runs 50-200 shell commands, with outputs ranging from a few tokens (`ls`) to thousands of tokens (`cat` of a 500-line file, `cargo build` output). Most of these outputs contain boilerplate, ANSI escape codes, progress bars, and verbose formatting that wastes tokens without providing actionable information.

Existing solutions like RTK (Rust Token Killer) reduce token consumption by 60-90% through agent-level hook-based command rewriting and output filtering. However, RTK is stateless — it has no session awareness, no cross-command caching, and no ability to detect repeated reads of unchanged files.

baish takes a different approach: instead of hooking into the agent, baish **replaces bash entirely** with a minimal POSIX shell that understands command semantics, maintains session state, and optimizes execution at the shell level. This enables capabilities that a stateless proxy cannot provide:

- **Conversation cache**: track what the model has been told, emit "unchanged" markers for repeated reads.
- **Session-aware caching**: cache file hashes across commands, detect unchanged files without re-reading.
- **Plugin architecture**: per-command plugins that can execute with optimized flags, cache results, and fingerprint outputs.
- **Lossless transformations**: strip ANSI, compress JSON, remove git hints — all transformations that preserve semantic equivalence.

## What Changes

- **NEW**: baish binary — a minimal POSIX shell that replaces bash for AI agent sessions.
- **NEW**: Plugin system — compiled Rust plugins for cat, git, cargo, curl, ls, grep, find, tree.
- **NEW**: Session store — SQLite-backed cache for file hashes and conversation history.
- **NEW**: Conversation cache — track what the model has been told, emit "unchanged" markers for repeated reads.
- **NEW**: Builtins — cd, export, unset, source, exit with full state tracking.
- **NEW**: `.bashrc` sourcing on startup — ensures user aliases, functions, and env vars are available.

## Capabilities

### New Capabilities

- `shell-execution`: baish reads commands from stdin, parses them, dispatches to plugins or execs real binaries, and writes output to stdout. The harness believes it's talking to bash.
- `plugin-cat`: cat plugin that caches file reads, detects unchanged files, and emits compact markers.
- `session-cache`: SQLite-backed session store that tracks file hashes and conversation history across commands within an AI_SESSION.
- `conversation-cache`: Track what the model has been told. On repeated reads of unchanged files, emit `[baish] <file> unchanged since last read` instead of repeating content.
- `builtin-cd`: cd builtin with full POSIX semantics (cd, cd -, cd ~, cd /path), tracks cwd in session.
- `builtin-source`: source builtin that reads and executes files through baish's dispatch, preserving state changes.
- `startup-rc`: Source `.bashrc` on startup to load user aliases, functions, and environment variables.

### Modified Capabilities

(none — greenfield project)

## Impact

- **New binary**: `baish` — single Rust binary, no runtime dependencies.
- **New directory**: `.baish/` at project root — session store database files.
- **No changes to existing code**: baish is a standalone project.
- **No changes to harness**: baish is transparent — the harness believes it's talking to bash.
