## 1. Extend data model

- [x] 1.1 Add `command TEXT`, `exec_time_ms INTEGER`, `cache_hit INTEGER` columns to `conversation_cache` CREATE TABLE and ALTER TABLE IF NOT EXISTS
- [x] 1.2 Add `command: Option<String>`, `exec_time_ms: Option<i64>`, `cache_hit: Option<bool>` fields to `ConversationEntry`
- [x] 1.3 Update `record_conversation()` signature and SQL INSERT to accept and store new fields

## 2. Record command metadata during dispatch

- [x] 2.1 Pass command string to `record_conversation()` — truncate to 200 chars
- [x] 2.2 Record `Instant::now()` at dispatch start, compute elapsed at end, store as `exec_time_ms`
- [x] 2.3 Set `cache_hit = true` when `output_format == "unchanged"``

## 3. Extend gain report aggregation

- [x] 3.1 Add `total_saved_bytes`, `total_saved_tokens`, `total_raw_tokens`, `total_filtered_tokens` to `GainReport`
- [x] 3.2 Add `BypassEntry` struct, aggregate top N bypassed commands by call count
- [x] 3.3 Add `SequentialDup` struct, detect sequential duplicates by comparing consecutive commands
- [x] 3.4 Add `DayEntry` struct, aggregate savings by day from `first_shown` timestamps

## 4. Enhance gain report display

- [x] 4.1 Update `format_gain_report()` to show absolute bytes and estimated tokens
- [x] 4.2 Add "Top bypassed" section showing command, calls, cache hits, hit rate
- [x] 4.3 Add "Sequential duplicates" section (verbose mode only)
- [x] 4.4 Add time-series section with ASCII sparklines (daily/weekly)

## 5. Add CLI flags and verify

- [x] 5.1 Add `--daily` and `--weekly` flags to `Args` struct in `main.rs`
- [x] 5.2 Wire flags to gain report — filter time-series data and pass to `format_gain_report()`
- [x] 5.3 Run `cargo test` to verify no regressions
