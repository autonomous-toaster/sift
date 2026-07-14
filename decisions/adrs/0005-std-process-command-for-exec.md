---
status: proposed
date: 2026-07-14
---

# std::process::Command for sift.exec() (replacing PTY)

## Context and Problem Statement

`sift.exec()` is the primary mechanism for plugins to run shell commands and capture output. The current implementation uses `portable-pty` to create a PTY, spawn bash, and read the PTY master. This causes two problems:

1. **Pager blocking**: Commands like `git diff` detect the PTY as a terminal and enable pagers (`less`), which block waiting for keypresses.
2. **Mixed stdout/stderr**: The PTY connects both stdout and stderr to the same fd — plugins cannot distinguish errors from output.

Should sift.exec() continue using a PTY, or switch to std::process::Command with pipes?

## Considered Options

* **PTY (current)** — portable-pty spawns bash in a PTY. Both stdout and stderr go to the PTY master. Requires PAGER=cat workaround for pager blocking.
* **std::process::Command with pipes** — Standard subprocess with piped stdout and stderr. Clean separation. No pager issues with TERM=dumb.
* **PTY for stdout, pipe for stderr** — Hybrid approach using unsafe pre_exec to redirect stderr to a pipe. Complex, requires raw fd access.

## Decision Outcome

Chosen option: **std::process::Command with pipes**, because:

- sift is a non-interactive proxy for AI agents — no command needs a TTY.
- Pipes give clean stdout/stderr separation at zero complexity cost.
- No pager blocking — git, less, etc. see a pipe and skip pagination.
- No ANSI color codes — TERM=dumb ensures plain text output.
- Removes dependency on portable-pty and its transitive dependencies.
- Simpler code — std::process::Command is the standard Rust API for subprocesses.

### Consequences

* Good, because stdout and stderr are captured separately — plugins can distinguish errors from output.
* Good, because pager blocking is eliminated without workarounds.
* Good, because portable-pty dependency can be removed, reducing compile time and dependency surface.
* Good, because std::process::Command is well-understood, well-documented, and cross-platform.
* Bad, because programs that check isatty() for behavior changes (e.g., color output, progress bars) will see a pipe. This is desirable for AI agents (plain text is preferred), but may surprise users running sift interactively.
* Bad, because interactive commands (top, less, vim) will not work through sift.exec(). This is acceptable — AI agents do not run interactive commands.

## Environment contract

Every process spawned by sift.exec() receives:

| Variable | Value | Reason |
|---|---|---|
| `PAGER` | `cat` | Prevent pager blocking (git diff, less, etc.) |
| `TERM` | `dumb` | Prevent ANSI color codes, disable TTY-specific features |
| `EDITOR` | `true` | Prevent editor blocking (git rebase --continue, etc.) |
| `GIT_EDITOR` | `true` | Belt-and-suspenders for git |
| `GIT_PAGER` | `cat` | Belt-and-suspenders for git |

These are set via `std::process::Command::env()`. The child process inherits all other environment variables from the parent.
