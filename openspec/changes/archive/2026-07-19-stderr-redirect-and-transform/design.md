## Context

`exec_command` spawns bash with stdout and stderr pipes. Fd redirects like `2>&1` are handled by bash internally — both streams go to the stdout pipe. The transform is applied to stdout chunks as they're read. Stderr content (progress bars, errors) gets transformed alongside actual output.

## Goals / Non-Goals

**Goals:**
- Detect all `N>&M` fd redirects in args, strip before passing to bash
- Handle `2>&1` (stderr→stdout) — keep streams separate, apply transform to stdout only, append stderr raw
- Handle `1>&2` (stdout→stderr) — strip from args, let bash handle naturally
- Pipeline deferral: if `|` detected, skip all redirect handling — let bash handle it
- Fix bash.lua to return output explicitly

**Non-Goals:**
- Changing `> file` / `>> file` behavior (file gets whatever plugin outputs)
- Handling `N>file` redirects (already handled by `dispatch_with_redirect`)
- New plugin system or stream abstraction
- Changes to non-bash plugins

## Decisions

1. **Generic fd redirect parsing** — `parse_fd_redirects()` checks for args matching `/^\d+>&\d+$/`. Returns a map of fd→target. Currently handles `2>&1` and `1>&2`. Extensible to other fds.

2. **Arg-level detection** — `dispatch_with_redirect` calls `parse_fd_redirects()` on args, strips matched args, sets `merge_stderr` flag for `2>&1`.

3. **Flag flows through dispatch** — The `merge_stderr` flag is passed from `dispatch_with_redirect` to `dispatch` to the plugin. The plugin passes it to `sift.exec` as an option. `sift.exec` passes it to `exec_command`.

4. **exec_command handles separation** — When `merge_stderr = true`, `exec_command` spawns bash without `2>&1`, captures stdout/stderr separately, applies transform to stdout chunks, then returns `(transformed_stdout + stderr, stderr, exit_code)`.

5. **Pipeline deferral** — `dispatch_full` already detects pipes via `try_pipeline()`. When a pipe is detected, sift runs preceding segments in bash and pipes to the last segment's plugin. Bash handles all redirects naturally. No sift interception needed.

6. **`> file` unchanged** — File redirects write whatever the plugin outputs (transformed or not). The user chose to use sift.

7. **bash.lua returns output with streamed flag** — Currently bash.lua returns `{ status, exit_code }` without `output`, relying on `exec_command` printing to stdout. Fix: return `output` in the result table with `streamed = true`. When `streamed = true`, `dispatch` skips printing the output (it was already streamed by `exec_command`). This preserves real-time streaming while giving the gain recording the output it needs.

## Risks / Trade-offs

1. **Edge cases** — `2>&1` inside quotes (`echo "2>&1"`) or combined with other redirects (`2>&1 > file`). → Mitigation: exact match on arg pattern, not substring search.

2. **Streaming preserved** — `exec_command` continues to stream output to stdout in real-time. The `streamed` flag tells `dispatch` not to print the returned output again, avoiding double-print without sacrificing streaming.