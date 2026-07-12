## Context

AI coding agents execute shell commands through a PTY connected to bash. The agent sends command strings, bash executes them, and the output is returned to the agent's context window. This output competes for context window space with the agent's reasoning and conversation history.

baish replaces bash in this pipeline. The harness spawns baish (named `bash` in PATH) instead of real bash. baish reads commands from stdin, parses them with brush-parser, classifies them, then either serves from cache (for simple file reads) or spawns real bash via a PTY for execution. PTY output flows through a streaming filter pipeline before reaching the agent.

## Architecture

```
Harness (pi, Claude Code)
    │  stdin/stdout
    ▼
┌──────────────────────────────────────────────────────┐
│  baish                                                │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ Parser   │  │ Classifier│  │ Plugin Registry  │   │
│  │ (brush-  │  │ FileRead  │  │ Rust (built-in)  │   │
│  │  parser)  │  │ Build     │  │ Lua (future)     │   │
│  │          │  │ Test      │  │ Subprocess       │   │
│  │          │  │ Git       │  │ (future)         │   │
│  │          │  │ Unknown   │  └──────────────────┘   │
│  └────┬─────┘  └─────┬─────┘                        │
│       │              │                               │
│       └──────┬───────┘                               │
│              ▼                                       │
│  ┌─────────────────────┐                            │
│  │ Decision Engine      │                            │
│  │                      │                            │
│  │ Simple FileRead?     │                            │
│  │   ├── Cache hit → emit marker, DONE              │
│  │   └── Cache miss → PTY                           │
│  │ Compound/Other?                                   │
│  │   └── PTY always                                  │
│  └─────────────────────┘                            │
│              │                                       │
│              ▼                                       │
│  ┌─────────────────────┐  ┌──────────────────────┐  │
│  │ PTY Manager         │  │ Output Pipeline      │  │
│  │ (portable-pty)      │  │ (StreamFilter)       │  │
│  │                     │  │                      │  │
│  │ real bash (child)   │  │ raw bytes → classify │  │
│  │                     │  │ → filter → agent     │  │
│  └─────────────────────┘  └──────────────────────┘  │
│              │                                       │
│              ▼                                       │
│  ┌─────────────────────┐                            │
│  │ Session Store       │                            │
│  │ (SQLite)            │                            │
│  │ ~/.baish/sessions.db│                            │
│  └─────────────────────┘                            │
└──────────────────────────────────────────────────────┘
    │  stdout (filtered for agent, full for human)
    ▼
Harness
```

## Decisions

### D1 — PTY proxy with parser classification

baish owns the PTY. The parser classifies commands before execution. Simple file reads can skip PTY entirely on cache hit. Compound commands always go through PTY for correctness, with post-execution output deduplication.

**State variable**: `execution_mode ∈ {direct, pty}`

### D2 — Two modes: agent and human

Agent mode (`-c command`): no TUI, filtered stdout, exit with code. Human mode (no `-c`): REPL with ratatui TUI showing agent view and raw output side by side.

**State variable**: `mode ∈ {agent, human}`

### D3 — Plugin system with priority

Plugin registry uses longest-prefix matching with priority tiebreaker. Built-in Rust plugins have priority -100. User plugins (Lua, subprocess) have priority 0-100. This allows overriding any built-in behavior.

**State variable**: `plugin_resolution ∈ {builtin, lua, subprocess}`

### D4 — Pre-compute hashes for cache-before-execution

For simple FileRead commands, hash the file before deciding to execute. If cache hit, skip PTY entirely. For compound commands, pre-compute hashes and pass to the filter for early output dedup.

**State variable**: `cache_strategy ∈ {pre_exec, post_exec}`

### D5 — Streaming output via StreamFilter trait

RTK-inspired StreamFilter trait with `feed_line()`, `flush()`, `on_exit()` methods. Each command class gets its own filter instance. Filters are stateful and produce per-command summaries.

**State variable**: `filter_state ∈ {streaming, buffered, summary}`

### D7 — Plugin command rewriting

Plugins can optionally rewrite a command before it reaches the PTY. The `Plugin` trait gains a `rewrite()` method that returns `Some(rewritten_command)` or `None` (keep original). This enables RTK-style optimization: `git status` → `git status --porcelain=v2`, `cargo test` → `cargo test --message-format=json`.

**State variable**: `command_source ∈ {original, rewritten}`

### D8 — Full output storage to temp files

When a filter truncates or summarizes output, the full raw PTY output is saved to `/tmp/baish/<session_id>/<timestamp>_<command_slug>.log`. The agent output includes a hint line with the path to the full file. Temp files are cleaned up on session end. This makes all optimizations lossless — no information is destroyed, just moved out of the context window.

**State variable**: `output_storage ∈ {inline, temp_file}`

Three crates: `baish-core` (traits, types, session store), `baish-filters` (StreamFilter implementations), `baish` (binary, PTY management, CLI). This enables `cargo crap` complexity checking and clean separation of concerns.

**State variable**: `crate_type ∈ {library, binary}`

## Goals / Non-Goals

**Goals:**
- Replace bash as the shell for AI agent sessions with a PTY-based proxy
- Stream output line-by-line through per-command-class filters
- Support two modes: agent (`-c`) and human (REPL with TUI)
- Provide a plugin system with priority-based resolution
- Pre-compute file hashes for cache-before-execution optimization
- Multi-crate workspace for code quality tooling

**Non-Goals:**
- Full bash compatibility — baish targets the subset used by AI agents
- Windows support — v1 targets Linux and macOS only
- Lua plugin implementation — design the trait, don't implement the runtime
- TUI polish — v1 TUI is functional, not feature-complete
