---
status: proposed
date: 2026-07-11
---

# Minimal POSIX Shell (not PTY Proxy, not LD_PRELOAD)

## Context and Problem Statement

What mechanism should baish use to intercept commands executed by the AI agent? The harness (pi, Claude Code, etc.) believes it's talking to bash. baish must intercept commands transparently without harness modifications.

## Considered Options

* Minimal POSIX shell — baish IS the shell. Parse input, dispatch to plugins or exec real binaries.
* PTY proxy — sit between harness and real bash, observe byte stream.
* LD_PRELOAD / DYLD_INSERT_LIBRARIES — hook execve/posix_spawn at the libc level.
* Agent hooks — use PreToolUse hooks (RTK approach).
* PATH shadowing — provide wrappers for each command.

## Decision Outcome

Chosen option: Minimal POSIX shell, as recommended by the final conclusion of the design exploration (see AI-optimized Shell Design PDF, page 43-44). The parser gives semantic awareness that a PTY proxy can never have — distinguishing `cat foo` from `cat foo | grep bar`, understanding redirections, and making optimization decisions before any process is launched. No harness modifications, no fragile libc hooks, no SIP issues on macOS.

### Consequences

* Good, because the parser provides full semantic awareness of the command structure.
* Good, because it works on Linux and macOS without SIP issues.
* Good, because it's invisible to the harness — the harness believes it's talking to bash.
* Good, because it provides a clean place to implement session tracking, plugins, and output rewriting.
* Bad, because we must implement shell builtins (`cd`, `export`, `source`, `exit`) and source `.bashrc` on startup.
* Bad, because we must handle pipe setup between plugins and real binaries.
