## Context

sift has two execution modes: `agent_mode` (used by `sift -c`) and `repl_mode` (interactive). Both call `dispatch_full()` which returns `(output, exit_code, plugin_name)`. `repl_mode` prints the output; `agent_mode` discards it with `let (_output, ...)`.

The pipeline optimization in `try_pipeline()` runs the preceding command(s) via `Command::output()`, captures stdout+stderr, and either:
- Returns `(stdout+stderr, exit_code, "pipeline")` on non-zero exit (output goes to `agent_mode` → discarded)
- Dispatches the last segment to a plugin on success (output printed by `exec_command`)

Two secondary issues were discovered during investigation:
1. `try_pipeline` drops stderr when exit_code == 0
2. `exec_command` swallows EPIPE from `write_all` with `let _ =`

## Goals / Non-Goals

**Goals:**
- `sift -c 'grep ... file.txt | head -5'` MUST display output when the preceding command fails
- `sift -c` and `sift` (REPL) MUST behave consistently — both print output from `dispatch_full`
- stderr from the preceding command in a pipeline MUST be visible to the user
- EPIPE errors from stdin writes MUST NOT cause silent data loss

**Non-Goals:**
- No change to the pipeline optimization strategy (still runs preceding command fully before dispatching last segment)
- No change to the Lua plugin API
- No change to cache behavior

## Decisions

### Decision 1: Print output in `agent_mode` (same as `repl_mode`)

**Option A** (chosen): Change `let (_output, ...)` to `let (output, ...)` and print it, matching `repl_mode`.

**Option B**: Have `dispatch_full` always print output internally. Rejected because `dispatch_full` is a library function that returns data — printing is the caller's responsibility.

**Rationale**: Simplest fix, consistent with existing pattern, no behavioral change for the success path (output is already printed by `exec_command`).

### Decision 2: Forward stderr from preceding command on exit 0

**Option A** (chosen): Always include stderr in the output passed to the last segment's plugin, not just on non-zero exit.

**Option B**: Print stderr separately to stderr. Rejected because the pipeline optimization captures the preceding command's output — stderr should be part of the pipeline's output.

**Rationale**: If grep writes a warning to stderr (e.g., "binary file matches"), the user should see it. The current code only forwards stderr on non-zero exit, which is inconsistent.

### Decision 3: Handle EPIPE from `write_all`

**Option A** (chosen): Check the result of `write_all` and `flush`. If EPIPE, log a debug message and continue — the child has already read what it needs.

**Option B**: Use threads to read stdout concurrently while writing stdin. Rejected as over-engineering for a case that doesn't manifest in practice.

**Rationale**: The EPIPE case is theoretically possible but was not observed in testing with 500MB inputs. A simple error check with a debug log is sufficient.

## Risks / Trade-offs

- **[Risk] Forwarding stderr on exit 0 could change behavior for plugins that expect clean stdin.** Mitigation: stderr is appended to the stdin string, so plugins see it as part of their input. This is consistent with how bash pipelines work (stderr from preceding commands is visible).
- **[Risk] The EPIPE fix adds a log line that could be noisy.** Mitigation: use `tracing::debug!` level, not `warn!` or `error!`.
- **[Risk] No test for the EPIPE case.** Mitigation: the EPIPE case is hard to reproduce reliably. The fix is defensive (check error, log, continue) and low-risk.
