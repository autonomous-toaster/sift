## Why

The `sift --gain` report currently shows only reduction percentage per plugin. This is insufficient for understanding actual token savings, identifying wasteful patterns, and debugging plugin behavior. Users need absolute byte and token counts, per-command breakdowns, and time-series data to make informed decisions about their workflow.

### Problems

1. **Percentage-only display is misleading** — A plugin showing 80% reduction might save 10 bytes or 10 MB. Without absolute values, users can't distinguish meaningful savings from noise.

2. **No per-command visibility** — The report shows "cat.lua: 15 calls, 82.3% reduction" but not which specific files were read repeatedly. A user can't tell if `cat Cargo.toml` was read 10 times (wasteful) or 10 different files were read once (efficient).

3. **No bypass/cache-hit breakdown** — The report counts bypasses but doesn't show which commands were bypassed or how often the same command was repeated. Sequential duplicates (e.g., `cat foo.rs` then `cat foo.rs` again) are invisible.

4. **No time-series data** — Users can't see how savings trend over time. Was yesterday more efficient than today? Are changes to plugins improving or degrading performance?

5. **No token estimation** — The report uses basis points (1/100 of a percent) but doesn't display estimated token counts, which is what users actually care about for LLM cost analysis.

### Non-Goals

- Real tokenizer integration (use chars/4 heuristic)
- Full command history browser (keep it as a CLI report)
- Real-time monitoring (keep it as a post-hoc analysis tool)
- Multi-session cross-referencing (keep it per-session)

## What

Add to the `--gain` report:

- **Absolute byte and token savings**: "Raw: 1.2 MB → Filtered: 340 KB (71.7%, 880 KB saved, ~220K tokens)"
- **Per-command breakdown**: Top N most-bypassed commands with call count, cache hit rate
- **Sequential duplicate detection**: Commands repeated back-to-back (opt-in, verbose mode)
- **Time-series data**: Savings by day/week with ASCII sparklines
- **New CLI flags**: `--daily`, `--weekly`, `--verbose`, `--json` (already exists)

Data model changes:
- Add `command TEXT` column to `conversation_cache` (truncated to 200 chars, recorded AFTER `cd` prefix peeling so `cd /long/path && cmd` stores just `cmd`)
- Add `exec_time_ms INTEGER` column
- Add `cache_hit BOOLEAN` column
