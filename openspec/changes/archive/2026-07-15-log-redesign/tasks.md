## 1. Log level methods

- [x] 1.1 Rewrite `register_log` to register `sift.log.{info,warn,error,debug}` as level methods instead of a callable table with metatable. Remove `__call` metatable. Each method accepts `(ctx, msg)` and prints `[sift] LEVEL: msg` to stdout/stderr as before.
- [x] 1.2 Update tests in `lua/mod.rs` to use new level method signatures instead of callable `sift.log(ctx, level, msg)`.

## 2. Nudge as top-level

- [x] 2.1 Create `register_nudge` method that registers `sift.nudge(ctx, msg)` as a standalone function on the `sift` table, capturing `self.nudges.clone()` and pushing to the accumulator.
- [x] 2.2 Update `register_sift_table` to call `register_nudge` and remove `sift.log.nudge` registration from `register_log`.
- [x] 2.3 Update tests to use `sift.nudge` instead of `sift.log.nudge`.

## 3. Cleanup

- [x] 3.1 Verify `just ci` passes with all changes.
