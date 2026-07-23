## Context

`sift --gain` currently queries `conversation_cache` and aggregates by plugin. The `GainReport` struct holds `total_commands`, `total_raw_bytes`, `total_filtered_bytes`, `reduction_bps`, `bypass_count`, and `per_plugin`. The `format_gain_report` function renders this as a human-readable table.

When no `AI_SESSION` is set, `query_conversations(None)` returns all entries — but the output doesn't indicate this. The report also lacks temporal context and absolute savings.

## Goals / Non-Goals

**Goals:**
- Show session count when displaying all sessions
- Show date range of the data
- Show total tokens saved in absolute terms (KB)
- Zero new CLI flags, zero new dependencies, zero database schema changes

**Non-Goals:**
- No new filtering or sorting options
- No JSON output from CLI (already available via Lua API)
- No historical trend analysis
- No per-session breakdown in all-sessions mode

## Decisions

**Decision 1: Derive session count from item_id prefixes**
The `item_id` format is `{session_id}_{invocation_id}_{cmd_count}`. The session ID is the prefix before the first `_`. Count unique prefixes across all entries. This is O(n) during the existing iteration in `generate_gain_report`.

**Decision 2: Track min/max timestamps during iteration**
`ConversationEntry` already has `first_shown` and `last_shown` (unix ms). Track the minimum `first_shown` and maximum `last_shown` across all entries during the existing loop. No extra queries needed.

**Decision 3: Add optional fields to GainReport**
Add `session_count: Option<i64>`, `first_seen: Option<i64>`, `last_seen: Option<i64>` to `GainReport`. These are `None` when a specific session is filtered (since the scope is already clear). Only populated when `session_id` is `None` in `generate_gain_report`.

**Decision 4: Format date range in local time**
Use `chrono` (already a dependency) to convert unix ms timestamps to `YYYY-MM-DD` format. Single-day range shows one date, multi-day shows `YYYY-MM-DD – YYYY-MM-DD`.

## Risks / Trade-offs

- [Session count derivation] → Relies on `item_id` format convention. If format changes, count breaks. Mitigation: trivial to update alongside any future format change.
- [Timestamp accuracy] → `first_shown`/`last_shown` are set at record time, not command execution time. For gain tracking this is fine — the period reflects when data was recorded, not when commands ran.
- [No per-session breakdown] → User can't see which session contributed what. Mitigation: out of scope by design. The all-sessions view is a summary. Per-session detail is available by setting `AI_SESSION`.
