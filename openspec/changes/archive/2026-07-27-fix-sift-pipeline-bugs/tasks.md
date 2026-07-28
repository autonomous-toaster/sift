## 1. Fix stdout flush in agent mode

- [x] 1.1 Add `std::io::stdout().flush()` before `std::process::exit()` in `sift/src/main.rs` and in `agent_mode()` after `dispatch_full()` returns
- [x] 1.2 Verify: `sift -c "echo '{\"a\":1}' | jq '.a'"` produces visible output `[1]` with exit code 0

## 2. Make `split_pipeline` quote-aware

- [x] 2.1 Replace the naive character-by-character `|` splitter in `sift-core/src/lua/api.rs` with a state machine that tracks single quotes, double quotes, and backslash escapes
- [x] 2.2 Add unit tests for `split_pipeline()` covering: pipe inside single quotes, pipe inside double quotes, escaped pipe, pipe outside quotes, `||` logical OR, empty segments
- [x] 2.3 Verify existing pipeline tests still pass: `test_split_pipeline_simple`, `test_split_pipeline_logical_or`, `test_split_pipeline_no_pipe`

## 3. Add pipeline fallback for unmatched segments

- [x] 3.1 In `try_pipeline()`, when `find_plugin()` returns `None`, run the entire pipeline through `exec_command()` in bash instead of returning `None`
- [x] 3.2 Add unit test `test_pipeline_fallback_to_bash` that verifies `echo hello | grep hello` produces correct output via bash fallback
- [x] 3.3 Verify existing pipeline tests still pass: `test_pipeline_triggers_for_specific_plugin`, `test_pipeline_skips_wildcard_plugin`, `test_jq_plugin_basic_filter`

## 4. Verify regression coverage

- [x] 4.1 Run full test suite: `cargo test -p sift-core` — all tests MUST pass
- [x] 4.2 Run `sift -c "echo '{\"a\":1}' | jq '.a'"` in agent mode — output MUST be `[1]`, exit code 0
- [x] 4.3 Run `sift -c "echo hello | cat"` in agent mode — output MUST contain `hello`
- [x] 4.4 Run `sift -c "echo hello | grep hello"` in agent mode — output MUST contain `hello`
