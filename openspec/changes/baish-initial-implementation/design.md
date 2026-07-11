## Context

AI coding agents execute shell commands through a PTY connected to bash. The agent sends command strings, bash executes them, and the output is returned to the agent's context window. This output competes for context window space with the agent's reasoning, tool calls, and conversation history.

baish replaces bash in this pipeline. The harness spawns baish instead of bash. baish reads commands from stdin, parses them with brush-parser (a full POSIX/bash AST parser), and decides per-command whether to:

1. **Run a plugin** — if the command is interceptable and a plugin is registered.
2. **Exec the real binary** — if no plugin exists or the command is in a pipeline.
3. **Handle as a builtin** — for cd, export, source, exit.

Plugins can cache results, check fingerprints, and emit compact "unchanged" markers when the output is identical to a previous invocation. The session store (SQLite) tracks file hashes and conversation history across commands.

## Architecture

```
Harness (pi, Claude Code, etc.)
    │  stdin/stdout (PTY)
    ▼
┌──────────────────────────────────────┐
│  baish                                │
│                                      │
│  ┌──────────┐  ┌──────────────────┐  │
│  │ Parser   │  │ Session Store    │  │
│  │ (brush-  │  │ (SQLite via sqlx)│  │
│  │  parser) │  └──────────────────┘  │
│  └────┬─────┘                        │
│       ▼                              │
│  ┌──────────┐                        │
│  │ Dispatcher│                       │
│  └────┬─────┘                        │
│       │                              │
│  ┌────┴────────┐                    │
│  │              │                    │
│  ▼              ▼                    │
│ Plugin        Exec real              │
│ (cat, git,    binary                 │
│  cargo...)    (/bin/cat,             │
│               /usr/bin/git)         │
│  │              │                    │
│  └──────┬───────┘                    │
│         ▼                            │
│  ┌──────────┐                       │
│  │ Output    │                       │
│  │ Pipeline  │                       │
│  └──────────┘                        │
└──────────────────────────────────────┘
    │  stdout
    ▼
Harness
```

## Decisions

### D1 — Interception rule: only optimize when stdout goes to PTY

A plugin is only invoked when the command's stdout goes directly to the harness (PTY). If stdout is piped to another command or redirected to a file, the plugin falls through to exec the real binary. This is determined by the parser, which knows the pipeline structure.

**State variable**: `interception_mode ∈ {direct, piped, redirected}`

### D2 — Pipeline handling: delegate to bash when no plugins involved

If a pipeline contains no plugin-invoked commands, delegate the entire pipeline to `/bin/bash -c "original command"`. If at least one command is a plugin, set up pipes manually in Rust.

**State variable**: `pipeline_mode ∈ {bash_delegated, rust_managed}`

### D3 — Session store: SQLite, one file per session

The session store lives at `.baish/session_<AI_SESSION_ID>.db`. Created on demand when `AI_SESSION` is set. Two tables: `file_cache` (path, hash, mtime) and `conversation_cache` (item_type, item_id, commands_since_at_create). No content blobs — file content is re-read from the filesystem.

**State variable**: `session_state ∈ {active, inactive}`

### D4 — Conversation cache staleness: command-count heuristic

After 50 commands since a cache entry was created, treat it as stale and show full content. The "unchanged" message always includes the staleness count so the model can decide whether to re-read.

**State variable**: `cache_entry_state ∈ {fresh, stale}`

### D5 — Startup: source .bashrc

On startup, if `~/.bashrc` exists, source it through baish's dispatch. This ensures user aliases, functions, and env vars are available. Uses brush-parser to parse the file as a full script (handles multi-line constructs).

**State variable**: `startup_state ∈ {rc_sourced, rc_not_found, rc_error}`

## Goals / Non-Goals

**Goals:**
- Replace bash as the shell for AI agent sessions with a minimal POSIX shell.
- Provide a plugin system for per-command optimization (cat first, then git, cargo, etc.).
- Provide a session store for cross-command caching and conversation tracking.
- Source `.bashrc` on startup for user environment compatibility.
- Implement shell builtins (cd, export, unset, source, exit) with full state tracking.

**Non-Goals:**
- Full bash compatibility — baish targets the subset of bash used by AI agents (single commands, simple pipelines, basic builtins). Complex scripts should be delegated to `/bin/bash`.
- Plugin ecosystem — v1 ships with compiled-in plugins. Third-party plugins are out of scope.
- Windows support — v1 targets Linux and macOS only.
- Performance benchmarking — v1 focuses on correctness. Optimization comes after.
