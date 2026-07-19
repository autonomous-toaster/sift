## Context

Every Lua plugin in sift manually parses its arguments using ad-hoc while-loops with index tracking. This pattern is repeated across 8 plugins, consuming ~152 lines of boilerplate. Adding a new flag or positional argument requires restructuring the manual parser. The classifier already splits commands into `{name, args}` where `args` is a Lua table of strings — but there's no shared helper to turn that table into a structured result.

## Goals / Non-Goals

**Goals:**
- Provide a single `sift.args.parse(args, spec)` function that handles flag scanning, value consumption, type coercion, and error reporting
- Convert all 8 shipped plugins to use it, removing all manual parsers
- Support the full range of flag patterns used by current plugins: boolean flags, value flags, short count syntax, combined short flags, long flags with `=` separator, `--` end-of-flags marker
- Return `nil, error_string` on parse failure so the plugin can decide passthrough vs error

**Non-Goals:**
- Subcommand parsing (not needed yet — classifier handles `git commit` via pattern matching)
- Negation flags (`--no-fresh`) — overkill for current use cases
- Validation beyond type coercion (e.g., sed expression format is left to the plugin)
- Performance optimization — parsing is a one-time cost per invocation on small arg lists

## Decisions

### Rust implementation over Lua
The parser is implemented in Rust (mlua function) rather than Lua. Rationale: better string handling, proper error types, no Lua GC pressure for the spec table traversal, and easier to add features later. The spec is read once at parse time and the result table is returned to Lua.

### Return convention: `(Value::Table, Value::Nil)` or `(Value::Nil, Value::String)`
Two return values let the plugin distinguish "can't handle these args" (nil, nil) from "bad args" (nil, "error message"). The first case triggers passthrough, the second shows the error to the agent.

### Combined short flags: boolean-only
`-vs` splits into `-v -s` only when both are boolean flags. Value flags in combined form (`-n10`) are not supported — must use `-n 10`. This avoids ambiguity with short count syntax (`-10`).

### Short count: opt-in via `opts.short_count`
`-10` is treated as `n=10` only when `short_count = true`. Checked before combined flag splitting, so `-10` is never split into `-1` + `0`.

### `--` end-of-flags marker
Everything after `--` is treated as positional arguments, per POSIX convention. This allows plugins to handle args that look like flags (e.g., negative numbers).

### Unknown flag handling
When `opts.allow_unknown = false` (default), any unrecognized flag causes the parser to return `nil` (no error) — the plugin passthroughs. When `true`, unknown flags are silently skipped along with their value (if the next arg doesn't look like a flag).

## Risks / Trade-offs

- **[Breaking change]** All plugins must be converted at once. Old plugins with manual parsers will continue to work (they don't call `sift.args.parse()`), but the inconsistency remains until converted. → Mitigation: convert all 8 plugins in the same change.
- **[Edge case: -n10]** Value flags in combined form are not supported. If a future plugin needs `-n10` meaning `-n 10`, it would need `-n 10` instead. → Acceptable: no current plugin uses this pattern, and the workaround is trivial.
- **[Edge case: unknown flag with value]** When `allow_unknown = true`, the parser skips the next arg if it doesn't start with `-`. This heuristic can fail for edge cases like `-o -v` where `-v` is a boolean flag that looks like a flag but is actually the value of `-o`. → Acceptable: this is a known POSIX ambiguity; plugins that need precise handling can use `allow_unknown = false` and list all known flags.
