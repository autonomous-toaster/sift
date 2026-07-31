## Context

Sift is a Lua-plugin-based shell proxy that intercepts commands and optimizes output for LLM consumption. It has ~15 plugins covering file reads, HTTP, CI tools, and more. The core optimization mechanisms are:

- **File caching**: Returns "unchanged" instead of re-reading files
- **Diff emission**: Shows only changed lines when a file is re-read
- **Output compression**: mdmin for markdown, TOON for JSON, rtk for git/docker/ls
- **Command bypass**: `command` prefix forces passthrough to bash

Analysis of agent sessions revealed three classes of bugs:

1. **Heredoc crash**: `cat << 'EOF'` causes `sift.fs.stat()` to raise a Lua error, then the heredoc content gets executed as bash commands, corrupting files
2. **Diff emission errors**: Agent receives a diff instead of full content, makes wrong edits, requires retry — costing more tokens than the diff saved
3. **Stale cache**: Agent reads a file, gets "unchanged" from cache, but the file was modified by a previous edit — leads to wrong edits and compilation errors

## Goals / Non-Goals

**Goals:**
- Prevent file corruption from heredoc syntax in cat.lua
- Eliminate agent errors caused by diff emission in sift-read.lua
- Ensure cache freshness via mtime checking and write invalidation
- Guarantee `command` prefix always bypasses all plugins

**Non-Goals:**
- No changes to mdmin, TOON, rtk, or other compression plugins
- No changes to the caching data model (SQLite schema stays the same)
- No changes to the Lua plugin API surface (except `sift.fs.stat()` return type)

## Decisions

### 1. Heredoc handling: path validation before stat

**Decision**: Add a path validation check in cat.lua before calling `sift.fs.stat()`. If the path contains shell metacharacters (`<`, `>`, `|`, `;`, `&`, `` ` ``) or doesn't exist, passthrough to bash.

**Alternatives considered**:
- **Catch Lua error in cat.lua**: Would work but every plugin would need the same fix — fragile
- **Fix in sift-core dispatch**: Detect heredoc syntax before dispatching to plugins — complex, shell parsing is hard
- **Path validation in cat.lua**: Simple, targeted, no side effects

**Why chosen**: The cat plugin is the only one affected by heredoc syntax. A targeted fix is safer than changing the dispatch layer.

### 2. `sift.fs.stat()` return nil on error

**Decision**: Change `sift.fs.stat()` to return `nil` instead of raising a Lua error when the path doesn't exist.

**Alternatives considered**:
- **Keep raising**: Every plugin must wrap stat in pcall — error-prone
- **Return `nil`**: Consistent with Lua conventions, all callers can check for nil

**Why chosen**: Defensive programming. Any plugin that calls `sift.fs.stat()` on a non-existent path would crash. Returning nil is safer and follows Lua conventions.

### 3. Remove diff emission from sift-read.lua

**Decision**: Remove the diff emission block entirely. On cache miss, always return full content with a one-line nudge: "file changed since last read".

**Alternatives considered**:
- **Keep diff with line numbers**: Still requires mental reconstruction — same errors
- **Annotated full content**: Full content with `+`/`-` markers — more tokens than full content alone, marginal benefit
- **Remove diff entirely**: Simplest fix, eliminates the error class entirely

**Why chosen**: The diff optimization is false economy. It saves tokens on reads but costs more in retries and error recovery. The cache already handles the "no change" case efficiently.

### 4. mtime-based cache invalidation

**Decision**: Before returning "unchanged", check the file's mtime. If mtime > last_read, force a re-read even if the content hash matches.

**Alternatives considered**:
- **Hash-only cache**: Current behavior — misses edits that don't change content hash (impossible for different content, but possible if file is reverted)
- **mtime + hash**: Catches all cases where file was modified

**Why chosen**: mtime is cheap (one stat call) and catches all modification cases. The hash is still used for content identity.

### 5. Cache invalidation on write

**Decision**: When `sift.fs.write()` or `sift.fs.edit()` is called, invalidate the path cache entry for that file.

**Alternatives considered**:
- **Don't invalidate**: Rely on mtime check on next read — works but wastes a stat call
- **Invalidate on write**: Proactive, no stale reads possible

**Why chosen**: Proactive invalidation is cheap and eliminates the window between write and next read where the cache is stale.

### 6. `command` plugin priority boost

**Decision**: Give the built-in `command` plugin a higher priority than all user plugins.

**Alternatives considered**:
- **Special-case in dispatch**: Check for `command` prefix before plugin matching — more complex
- **Priority boost**: Simple, uses existing priority system

**Why chosen**: The priority system already handles this. A higher priority ensures `command` prefix always matches before any other plugin.

## Risks / Trade-offs

- **[Token cost] Removing diff emission increases token consumption on file re-reads** → Mitigation: The cache still prevents re-reads of unchanged files. The diff was only emitted on change, which is a small fraction of reads. The token cost increase is bounded by the file size.
- **[Compatibility] `sift.fs.stat()` returning nil may break plugins that expect a table** → Mitigation: Audit all existing plugins. Only cat.lua and sift-read.lua call `sift.fs.stat()`. Both will be updated.
- **[Performance] mtime check adds a stat call per read** → Mitigation: stat is cheap (~microsecond). The existing code already calls `sift.fs.stat()` on every read.
