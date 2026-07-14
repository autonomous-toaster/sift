---
status: proposed
date: 2026-07-14
---

# Stderr Capture via Pipes, sift.exec() Returns (stdout, stderr, exit_code)

## Context and Problem Statement

`sift.exec()` currently returns `(output, exit_code)` where `output` is the combined stdout+stderr from the PTY. Plugins cannot distinguish errors from output. This means:

1. Error messages are mixed with normal output — plugins can't tell if a command succeeded or failed based on output alone.
2. Token optimization can't target stderr separately — warnings and errors count against the same budget as useful output.
3. Caching is imprecise — if stderr changes between runs (different timestamps, different warnings), the cache misses even though stdout is identical.

How should sift.exec() expose stderr to plugins?

## Considered Options

* **Combined output (current)** — Return stdout+stderr mixed. Plugins parse the combined string to find errors.
* **Separate return values** — sift.exec() returns `(stdout, stderr, exit_code)`. Breaking change but clean.
* **Combined output + sift.meta.stderr** — Return combined for convenience, expose stderr via sift.meta. Two ways to access the same data.
* **Combined output only** — Plugins use exit_code to detect errors, ignore stderr separation.

## Decision Outcome

Chosen option: **Separate return values** — `sift.exec(cmd) → stdout, stderr, exit_code`, because:

- It's the simplest, most explicit API — one call, three return values, no ambiguity.
- Plugins that don't care about stderr can ignore it: `local output, _, code = sift.exec(cmd)`.
- Plugins that need stderr can use it directly: `local output, err, code = sift.exec(cmd)`.
- No need for sift.meta.stdout/sift.meta.stderr — the data is in the return values.
- Backward compatibility is not a concern — we are building, not maintaining.

### Consequences

* Good, because every plugin has explicit access to stdout, stderr, and exit_code.
* Good, because the Lua idiom `local out, _, code = sift.exec(cmd)` makes ignoring stderr ergonomic.
* Good, because caching can be based on stdout only — stderr changes don't invalidate the cache.
* Good, because token optimization can target stderr separately (e.g., strip known warning patterns).
* Bad, because all existing plugins that call sift.exec() need to update their call sites. Acceptable — there are only 3 call sites (bash.lua, git_status.lua, and any user plugins).

## Plugin migration

```lua
-- Before:
local output, exit_code = sift.exec("git status --porcelain=v2")

-- After:
local output, stderr, exit_code = sift.exec("git status --porcelain=v2")
-- stderr is available if needed, or use _ to ignore:
local output, _, exit_code = sift.exec("git status --porcelain=v2")
```
