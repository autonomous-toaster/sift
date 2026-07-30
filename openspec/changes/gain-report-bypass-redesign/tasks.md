## 1. Fix passthrough recording

- [x] 1.1 Add `plugin_name: Option<&str>` parameter to `handle_passthrough_status`. Pass `entry.patterns.first().map(String::as_str)` from the dispatch call site. Store it in `record_conversation` instead of the hardcoded `Some("command")`.
- [x] 1.2 Compute actual command name in `handle_passthrough_status`: if `cmd == "command"` and args is non-empty, use `args[0]`; otherwise use `cmd.to_string()`. Store this as the `command` field instead of `cmd.to_string()`.

## 2. Add aggregation logic

- [x] 2.1 Add a helper function `command_name(entry: &ConversationEntry) -> String` that extracts the first whitespace-separated word from `entry.command`, with a fallback to `"unknown"` if command is None or empty.
- [x] 2.2 Add a `total_calls` aggregation pass in `generate_gain_report`: iterate all entries, extract command name, count total calls per command. Store in a `HashMap<String, i64>`.
- [x] 2.3 Change `BypassEntry` struct: replace `cache_hits: i64` and `hit_rate: f64` with `total_calls: i64` and `reason: String`. Update the bypass aggregation loop to look up total calls and determine reason (`"explicit"` if `plugin_name == "command"`, `"passthrough"` otherwise).
- [x] 2.4 Add `unmatched_entries: Vec<UnmatchedEntry>` to `GainReport`. Create `UnmatchedEntry { command: String, calls: i64 }`. In the aggregation loop, count entries where `plugin_name == "__default__"`, grouped by command name. Sort by calls descending, truncate to 10.

## 3. Update rendering

- [x] 3.1 Update `format_gain_report` "Top bypassed" rendering: show `command calls/total (percentage%) reason` instead of `command calls, cache hits (hit_rate% hit rate)`.
- [x] 3.2 Add "Commands without plugins" section rendering after the "Top bypassed" section: show `command calls` for each unmatched entry, with a header line. Only render if the list is non-empty.
