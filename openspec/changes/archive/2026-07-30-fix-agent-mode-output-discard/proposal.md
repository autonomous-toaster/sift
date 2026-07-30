## Why

When sift runs a piped command (e.g., `grep foo file.txt | head -5`) via `sift -c`, and the preceding command exits with a non-zero code, the output is silently discarded. The user sees no output at all — just an exit code. This happens because `agent_mode` in `main.rs` uses `let (_output, ...)` to discard the result from `dispatch_full`, while `repl_mode` correctly prints it.

Additionally, when the pipeline optimization succeeds (exit 0), stderr from the preceding command is silently dropped in `try_pipeline()`. And in `exec_command`, EPIPE errors from `write_all` are swallowed with `let _ =`, creating a latent data-loss risk.

## What Changes

- Fix `agent_mode` in `sift/src/main.rs` to print the output from `dispatch_full`, matching `repl_mode` behavior
- Fix `try_pipeline` in `sift-core/src/lua/api.rs` to forward stderr from the preceding command even on success (exit 0)
- Fix `exec_command` in `sift-core/src/lua/exec.rs` to handle EPIPE from `write_all` gracefully instead of silently swallowing it
- Add a regression test for piped commands where the preceding command fails

## Capabilities

### New Capabilities
- `pipeline-error-output`: When a piped command's preceding segment fails, its stdout+stderr MUST be displayed to the user rather than silently discarded

### Modified Capabilities
*(No existing specs to modify — this is a new change)*

## Impact

- `sift/src/main.rs`: `agent_mode()` — one-line change to print output
- `sift-core/src/lua/api.rs`: `try_pipeline()` — forward stderr on exit 0
- `sift-core/src/lua/exec.rs`: `exec_command()` — handle EPIPE from `write_all`
- `sift/tests/cli.rs`: new regression test for pipeline error output
