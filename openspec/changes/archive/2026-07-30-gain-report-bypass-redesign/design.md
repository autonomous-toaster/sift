## Context

The `sift gain` report aggregates conversation entries from the SQLite session store. Each entry has a `plugin_name`, `output_format`, `command`, and `cache_hit` field. The current "Top bypassed" section filters entries where `output_format == "passthrough"` and shows cache hit rate — which is always 0% because passthrough commands never set `cache_hit = true`.

Two data quality issues hide information:
1. `handle_passthrough_status` hardcodes `plugin_name = "command"` for ALL passthroughs, making it impossible to distinguish explicit bypasses (`command` prefix) from plugin-initiated passthroughs
2. The `command` field stores the first word (`"command"`) not the actual command (`"head"`), so explicit bypasses lose the real command name
3. `__default__` entries (1470 calls, 0% reduction) are invisible because they use `output_format = "text"`, not `"passthrough"`

## Goals / Non-Goals

**Goals:**
- Replace cache hit rate with bypass ratio (bypassed / total calls) in the "Top bypassed" section
- Show bypass reason: "explicit" (user typed `command` prefix) vs "passthrough" (plugin returned passthrough)
- Add a "Commands without plugins" section for `__default__` entries, aggregated by command name
- Fix conversation recording to store real plugin name and actual command for passthroughs

**Non-Goals:**
- Recording conversation entries for the invisible passthrough paths in `dispatch_full` (compound commands with `;`/`&&`/`||`) — that's a separate change
- Adding new database columns or schema migrations
- Changing the "Per plugin" section behavior

## Decisions

### Decision 1: Fix `handle_passthrough_status` to accept real plugin name

Instead of hardcoding `plugin_name = "command"`, accept `plugin_name: Option<&str>` from the caller (the dispatch function has access to `entry.patterns.first()`).

This means:
- `command head -n 10` → `plugin_name = "command"` (explicit bypass)
- `head -n 10` (piped) → `plugin_name = "head"` (plugin passthrough)

The reason is inferred: `plugin_name == "command"` → explicit, otherwise passthrough.

### Decision 2: Store actual command name, not first word

For `command head -n 10`, store `command = "head"` instead of `command = "command"`. The actual command is `args[0]` when `cmd == "command"` and args is non-empty. For other plugins, `cmd` is already the command name.

### Decision 3: Compute total calls by extracting command name from all entries

For all entry types, extract the command name as `command_field.split(' ')[0]` (first whitespace-separated word). This works for:
- Handled entries: `"git 'status'"` → `"git"`
- Passthrough entries (after fix): `"head"` → `"head"`
- `__default__` entries: `"git 'status'"` → `"git"`

### Decision 4: Separate sections for bypassed vs unmatched

Two independent sections:
- "Top bypassed": entries with `output_format == "passthrough"`, showing ratio + reason
- "Commands without plugins": entries with `plugin_name == "__default__"`, showing count only

A command can appear in both sections (e.g., `python3` may have both explicit bypasses and `__default__` calls).

## Risks / Trade-offs

- **Extracting command name from `command_field.split(' ')[0]`** is a heuristic. If a future change stores command names differently, this breaks. Mitigation: the extraction is localized in one function.
- **Invisible passthroughs remain invisible**. Compound commands with `;`/`&&`/`||` still leak passthrough executions that never appear in stats. This is a known gap but out of scope.
- **The `*` wildcard entries (346 calls)** are historical and will disappear on stats reset. Until then, they show as `plugin_name = "*"` in the per-plugin section but don't affect the new sections.
