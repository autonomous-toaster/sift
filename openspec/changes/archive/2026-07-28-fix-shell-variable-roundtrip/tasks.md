## 1. Thread original command through dispatch pipeline

- [x] 1.1 Add `original_cmd: &str` parameter to `dispatch_with_redirect()` in `sift-core/src/lua/api.rs` and pass it through to `dispatch()`
- [x] 1.2 Add `original_cmd: &str` parameter to `dispatch()` and pass it to `execute_passthrough()`
- [x] 1.3 Update `execute_passthrough()` to use `original_cmd` instead of reconstructing from `cmd + args`
- [x] 1.4 Update all callers of `dispatch()`, `dispatch_with_redirect()`, and `execute_passthrough()` to pass the original command

## 2. Bypass `__default__` plugin for original command

- [x] 2.1 In `dispatch()`, when the matching plugin is `__default__` (pattern == `__default__`), run the original command via `exec_command()` instead of calling the Lua plugin
- [x] 2.2 Ensure stdin is forwarded correctly when bypassing `__default__` plugin

## 3. Verify

- [x] 3.1 Run full test suite: `cargo test -p sift-core` — all tests MUST pass
- [x] 3.2 Verify `echo "\$HOME"` produces expanded path, not literal `$HOME`
- [x] 3.3 Verify existing pipeline tests still pass
- [x] 3.4 Verify jq plugin still works: `echo '{"a":1}' | jq '.a'` → `[1]`
