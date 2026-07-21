## Context

Sift's `try_pipeline()` optimizes pipelines by running preceding segments in bash and piping output to the last segment's plugin. Currently it triggers for ANY matching plugin, including wildcards (`rtk.lua` with `pattern = "*"`). When the wildcard plugin returns `passthrough` (e.g., `rtk` binary not found for `sort`), `execute_passthrough()` runs the command with empty stdin — the accumulated pipeline output is silently discarded.

The same path is hit when a specific plugin (like `head.lua`) returns `passthrough` or errors in pipeline mode, though the error case is already fixed by the dispatch wrapper nil guard.

## Goals / Non-Goals

**Goals:**
- Pipeline stdin is preserved when the last segment's plugin returns `passthrough`
- Wildcard plugins (`pattern = "*"`) do not trigger pipeline optimization
- No changes to the Lua plugin API or plugin behavior

**Non-Goals:**
- Not changing how plugins handle stdin internally (already addressed in separate fix)
- Not adding new plugin capabilities — only fixing the passthrough path

## Decisions

### Decision 1: Skip wildcard plugins in pipeline optimization

`try_pipeline()` checks `find_plugin()` which returns any match including wildcards. The fix: after finding the plugin, check if its pattern is `"*"` and return `None` (full bash pipeline) if so.

**Alternatives considered:**
- **Pass stdin through `execute_passthrough`** — necessary but insufficient alone. Wildcard plugins like `rtk.lua` are not designed for piped input; they delegate to `rtk` which doesn't know about the pipeline context. Better to skip optimization entirely.
- **Add a `pipeline_ok` flag to plugin metadata** — more flexible but adds API surface. The wildcard check is simpler and covers the common case.

### Decision 2: Forward stdin in `execute_passthrough`

`execute_passthrough()` currently hardcodes `stdin = ""`. Change it to accept an optional stdin string. When called from `dispatch()` in pipeline mode, the `StdinReader` content is read and passed through.

**Alternatives considered:**
- **Pipe the StdinReader directly to the subprocess** — more efficient but requires plumbing the reader through the exec layer. Reading to string is simpler and the data is already buffered from the preceding pipeline segments.

## Risks / Trade-offs

- **[Correctness]** Reading `StdinReader` to string in `dispatch()` buffers the entire pipeline output in memory. For very large pipelines this could increase memory usage. → Mitigation: the data is already buffered from the preceding `cmd.output()` call; this doesn't add new buffering.
- **[Edge case]** A specific plugin that legitimately returns `passthrough` in pipeline mode (e.g., `git-commit` with `-n` flag) will now correctly forward stdin. This is the desired behavior — no regression risk.
