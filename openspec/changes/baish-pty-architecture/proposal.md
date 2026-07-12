## Why

AI coding agents consume tokens for every shell command output. A typical session runs 50-200 commands, with outputs ranging from a few tokens to thousands. Most outputs contain boilerplate, ANSI codes, progress bars, and repeated content that wastes context window space.

The current baish implementation replaces bash with a minimal POSIX shell. This approach requires implementing shell semantics (parsing, builtins, pipelines) and cannot stream output — it buffers the entire command output before returning.

A PTY-based approach solves both problems: baish owns the PTY between the harness and real bash, enabling streaming output, two output channels (human gets full output, AI gets curated), and command classification via the parser without implementing shell execution.

## What Changes

- **REPLACE** the current shell-replacement architecture with a PTY-based architecture
- **NEW** PTY management: baish spawns real bash as a child process via a PTY
- **NEW** Streaming output pipeline: line-by-line filtering via StreamFilter trait
- **NEW** Command classification: parse with brush-parser, classify before PTY execution
- **NEW** Two modes: agent mode (`-c command`, no TUI) and human mode (REPL with TUI)
- **NEW** Plugin system with priority-based resolution, subcommand matching, and command rewriting
- **NEW** Full output storage: raw PTY output saved to temp files, agent gets path to full content
- **NEW** Pre-compute file hashes for cache-before-execution optimization
- **NEW** Session store at `~/.baish/sessions.db` (single DB, keyed by AI_SESSION)
- **KEEP** Existing CatPlugin, session store, builtins (cd, export, source)
- **KEEP** brush-parser for command parsing and classification

## Capabilities

### New Capabilities

- `pty-execution`: baish creates a PTY, spawns real bash, reads/writes the PTY, streams output
- `streaming-filter`: Output pipeline processes PTY output line-by-line, per-command-class filters
- `command-classification`: Parse input with brush-parser, classify as FileRead/Build/Test/Git/Unknown
- `agent-mode`: `bash -c 'command'` — no TUI, filtered stdout, exit with code
- `human-mode`: `bash` (no -c) — REPL with ratatui TUI showing agent view + raw output
- `plugin-priority`: Plugin registry with priority-based resolution and subcommand matching
- `plugin-rewrite`: Plugins can rewrite commands before execution (e.g., `git status` → `git status --porcelain=v2`)
- `full-output-storage`: Raw PTY output saved to `/tmp/baish/<session>/<timestamp>_<cmd>.log`, agent gets path
- `cache-before-exec`: Pre-compute file hashes, skip PTY for simple FileRead on cache hit
- `output-dedup`: Post-execution output hashing, emit "unchanged" marker for repeated output
- `multi-crate`: Workspace with baish-core (traits), baish-filters (filters), baish (binary)

### Modified Capabilities

- `plugin-cat`: CatPlugin now uses PluginContext instead of Session, supports override via priority
- `session-cache`: Session store moved to `~/.baish/sessions.db`, single DB for all sessions

## Impact

- **New binary**: `bash` (shadows real bash in PATH, knows real bash path)
- **New directory**: `~/.baish/` for session store
- **New temp directory**: `/tmp/baish/` for full output storage
- **New config**: `~/.config/baish/plugins/` for user plugins (future)
- **Dependencies added**: `portable-pty`, `tokio`, `ratatui` (human mode only)
- **Dependencies kept**: `brush-parser`, `sqlx`, `sha2`, `async-trait`
