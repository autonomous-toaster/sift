## Why

AI coding agents frequently run commands that leak sensitive environment variables into their output — `echo $AWS_SECRET_ACCESS_KEY`, `env`, `printenv`. These values appear verbatim in the agent's context, which may be logged, stored, or sent to LLM providers. Current sift plugins (rtk, cat, etc.) compress or cache output but do not redact secrets.

The existing scred binary already detects and redacts secrets in streaming text, but there is no sift plugin that routes command output through it. The agent has no way to safely inspect environment variables without risking credential exposure.

## What

A new sift plugin (`scred.lua`) that intercepts `echo`, `env`, and `printenv` commands, executes them normally, then pipes the output through the `scred` binary for secret redaction before returning it to the agent.

To support this, `sift.exec()` in the Rust Lua API gains a `stdin` option so plugins can pass captured output as stdin to a subprocess without temp files.

## Scope

- New plugin: `plugins/scred.lua` matching `echo`, `env`, `printenv`
- Rust change: add `stdin` option to `sift.exec()` in `sift-core/src/lua/api_reg_cache.rs`
- Plugin falls through to passthrough if `scred` is not installed or fails
- Only post-execution output redaction — no pre-execution env scrubbing

## Non-goals

- No changes to scred itself (used as an external binary)
- No pre-execution environment scrubbing (separate concern)
- No redaction of other commands (git, curl, etc.) — future work
- No changes to sift's pipeline splitting logic

## Outcome

An agent can safely run `echo $API_KEY` or `env` and see redacted output like `API_KEY=sk-***...` instead of the raw secret. The scred binary handles all detection and redaction logic — the plugin is a thin routing layer.
