## Why

Current nudge messages confuse AI agents with contradictory and ambiguous language. "unchanged (cached)" followed by "bypass if stale" plants doubt about cache accuracy. "raw: command git status" looks like metadata, not an instruction. Agents either misinterpret the intent or ignore the messages entirely.

Session log analysis showed **zero compliance** with nudge hints across 34 messages in a real session. The format is the problem — not the concept.

## What Changes

Rewrite all nudge messages across 6 plugins and 3 Rust locations to follow a consistent format:

`[nudge] <situation> — <action>`

Key changes:
- Remove "cached" from all messages (replaced with "unchanged" or "current")
- Remove "bypass if stale" (replaced with "to re-read")
- Remove standalone "raw:" prefix (kept as contextual `raw: command` inside the message)
- Remove feature installation instructions from binary document nudge
- Standardize burst warning format

## Capabilities

### New Capabilities
- `nudge-redesign`: Unified nudge message format across all plugins and Rust code

### Modified Capabilities
- (none)

## Impact

**Plugins (6 files):**
- `plugins/cat.lua` — 3 nudge messages
- `plugins/head.lua` — 2 nudge messages
- `plugins/tail.lua` — 2 nudge messages
- `plugins/sed.lua` — 2 nudge messages
- `plugins/sift-read.lua` — 5 nudge messages
- `plugins/rtk.lua` — 1 nudge message
- `plugins/jq.lua` — 2 nudge messages

**Rust (3 locations):**
- `sift-core/src/lua/api.rs` — burst warning nudge
- `sift-core/src/lua/api_reg_io.rs` — JSON shortest nudge, store nudge
- `sift-core/src/lua/api_reg_cache.rs` — error save nudge

**Tests:**
- `sift-core/src/lua/tests.rs` — update expected nudge strings
- `sift/tests/cli.rs` — update expected nudge strings
