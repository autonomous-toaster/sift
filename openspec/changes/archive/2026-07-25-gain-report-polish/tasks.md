## 1. Add formatting helpers

- [x] 1.1 Add `format_bytes()` with auto-scaling (B, KB, MB, GB)
- [x] 1.2 Add `format_int()` with thousand separators

## 2. Apply formatting in display

- [x] 2.1 Apply `format_bytes()` to raw/filtered/saved byte values in summary
- [x] 2.2 Apply `format_int()` to tokens, bps, and call counts
- [x] 2.3 Apply `format_bytes()` and `format_int()` to per-plugin lines

## 3. Data quality fixes

- [x] 3.1 Change `?` fallback to `(unknown)` in bypass aggregation
- [x] 3.2 Filter shell meta-commands (exit, cd, etc.) from bypass list
- [x] 3.3 Fix sparkline bar to show 0-width for 0 bytes

## 4. Verify

- [x] 4.1 Run `cargo test` — no regressions