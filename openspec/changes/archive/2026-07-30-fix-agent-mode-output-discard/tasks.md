## 1. Fix agent_mode output discard

- [x] 1.1 Change `let (_output, ...)` to `let (output, ...)` in `agent_mode()` and print output to stdout, matching `repl_mode` behavior
- [x] 1.2 Add regression test in `sift/tests/cli.rs` for `sift -c` with piped command where preceding command fails

## 2. Fix stderr forwarding in try_pipeline

- [x] 2.1 In `try_pipeline()`, include stderr in the output passed to the last segment's plugin even when exit_code == 0
- [x] 2.2 Add test for stderr visibility in pipeline output

## 3. Fix EPIPE handling in exec_command

- [x] 3.1 In `exec_command()` fast path, check the result of `write_all` and `flush` — if EPIPE, log at debug level and continue instead of silently swallowing
- [x] 3.2 Add test for EPIPE resilience (head with large stdin)
