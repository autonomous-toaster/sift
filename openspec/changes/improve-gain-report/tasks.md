## 1. GainReport struct changes

- [ ] 1.1 Add `session_count`, `first_seen`, `last_seen` optional fields to `GainReport` struct
- [ ] 1.2 Update `generate_gain_report` to track unique session count (from `item_id` prefix before first `_`) and min/max `first_shown`/`last_seen` during iteration
- [ ] 1.3 Populate new fields only when `session_id` is `None` (all-sessions mode)

## 2. format_gain_report rendering

- [ ] 2.1 Render session count in commands line: `"15 (across 3 sessions)"` when `session_count` is `Some`
- [ ] 2.2 Render date range as `"Period: YYYY-MM-DD – YYYY-MM-DD"` (single day: one date, multi-day: range) using `chrono` for timestamp formatting
- [ ] 2.3 Render absolute savings: append `", X KB saved"` to the reduction line

## 3. Verify

- [ ] 3.1 Build and run `sift --gain` without `AI_SESSION` — confirm session count, date range, and absolute savings appear
- [ ] 3.2 Build and run `sift --gain` with `AI_SESSION` — confirm new fields are absent (single-session mode unchanged)
- [ ] 3.3 Run `cargo test --release` — all tests pass
