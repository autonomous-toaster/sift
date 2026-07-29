## 1. Rust: add stdin option to sift.exec()

- [x] 1.1 Add `stdin` extraction from opts table in `sift-core/src/lua/api_reg_cache.rs` — extract `stdin` as `Option<String>` from the opts table and pass `stdin.as_deref()` to `exec_command()` instead of `None`
- [x] 1.2 Verify `exec_command()` in `sift-core/src/lua/exec.rs` already handles stdin correctly — the `stdin: Option<String>` parameter is already wired to write to the child's stdin pipe; confirm no changes needed

## 2. Plugin: create plugins/scred.lua

- [x] 2.1 Create `plugins/scred.lua` with pattern matching for `echo`, `env`, `printenv` — register plugin with `pattern = {"echo", "env", "printenv"}`, `priority = 0`, and `append_prompt` explaining redaction
- [x] 2.2 Implement `execute()` function — reconstruct command from `ctx.command` + args, execute via `sift.exec()`, capture output, pipe through `sift.exec("scred", {stdin = output})`, return redacted result
- [x] 2.3 Implement fallback logic — return `{status = "passthrough"}` when original command fails or output is empty; return original output when scred is not installed or fails

## 3. Tests

- [x] 3.1 Add unit test for `sift.exec()` stdin option in `sift-core/src/lua/tests.rs` — call `sift.exec()` with `{stdin = "test data"}` and `cat` as the command, verify output matches input
- [x] 3.2 Add integration test for scred plugin in `sift/tests/cli.rs` — run sift with scred plugin, execute `echo hello`, verify output is redacted; test fallback when scred is not in PATH
