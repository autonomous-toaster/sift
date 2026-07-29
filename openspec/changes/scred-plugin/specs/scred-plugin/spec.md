## ADDED Requirements

### Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `stdin` option to `sift.exec()` in Rust Lua API |
| T1.2 | Create `plugins/scred.lua` with echo/env/printenv matching |
| T1.3 | Add unit test for `sift.exec()` stdin option |
| T1.4 | Add integration test for scred plugin |

---

### Requirement: sift.exec() stdin option

`sift.exec()` SHALL accept an optional `stdin` field in its opts table. When provided, the string value SHALL be written to the subprocess's stdin pipe BEFORE the subprocess reads from its stdin.

#### SHALL: sift.exec() passes stdin to subprocess

T1.1 SHALL add a `stdin` field to the opts table of `sift.exec()`. CONCURRENTLY with T1.1, the Rust `exec_command()` function SHALL receive the stdin string and write it to the child process's stdin pipe before the child reads.

#### SHALL: stdin is optional and defaults to no input

T1.1 SHALL make the `stdin` field optional. IF the field is absent or nil THEN `sift.exec()` SHALL behave identically to the current behavior (no stdin written).

#### SHALL: stdin is a plain string

T1.1 SHALL accept only a Lua string for the `stdin` field. Binary data SHALL NOT be supported — callers SHALL encode binary as a string (lossy UTF-8) before passing.

---

### Requirement: scred plugin intercepts echo/env/printenv

A new plugin `plugins/scred.lua` SHALL intercept commands starting with `echo`, `env`, or `printenv`, execute them, and pipe the output through the `scred` binary for secret redaction.

#### SHALL: plugin matches echo, env, printenv

T1.2 SHALL register the scred plugin with `pattern = {"echo", "env", "printenv"}`. The plugin SHALL match any command whose first word is one of these three.

#### SHALL: plugin executes command and pipes through scred

T1.2 SHALL execute the matched command via `sift.exec()`. AFTER the command completes with exit code 0 and non-empty stdout, T1.2 SHALL pass the captured output as stdin to `sift.exec("scred", {stdin = output})`.

#### SHALL: plugin returns redacted output

T1.2 SHALL return the redacted output from scred as the command result. IF scred exits with code 0 THEN the plugin SHALL return `{status = "handled", output = redacted, exit_code = 0}`.

#### SHALL: plugin falls through on scred failure

T1.2 SHALL fall through to passthrough IF scred is not installed (exec returns non-zero). T1.2 SHALL also fall through IF the original command exits with non-zero or produces empty output.

#### SHALL: plugin has append_prompt

T1.2 SHALL set `append_prompt` to inform the agent that output is redacted and how to bypass with `command` prefix.

---

### Requirement: scred plugin is testable

The scred plugin SHALL have automated tests covering normal operation, scred absence, and edge cases.

#### SHALL: unit test for sift.exec() stdin

T1.3 SHALL add a unit test in `sift-core/src/lua/tests.rs` that calls `sift.exec()` with a `stdin` option and verifies the subprocess receives the data. The test SHALL use `cat` as the subprocess and verify the output matches the input.

#### SHALL: integration test for scred plugin

T1.4 SHALL add an integration test in `sift/tests/cli.rs` that runs sift with the scred plugin active, executes `echo hello`, and verifies the output is redacted. T1.4 SHALL also test the fallback behavior when scred is not in PATH.
