## Context

The current sift embeds all 5 plugins in the binary via `include_str!()`. The `sift.*` API has inconsistent ctx usage — only `sift.cache.*` takes ctx. JSON handling is limited to standard `serde_json::to_string/from_str`. There's no way to delegate commands to external tools, no smart token-minimization, and no unified nudge system. Error output is always saved regardless of exit code.

## Goals / Non-Goals

**Goals:**
- Split plugins into core (embedded) and optional (filesystem-loaded from `plugins/`)
- Add `sift.json.shortest()` for token-aware JSON representation selection
- Add `sift.log.nudge()` and automatic nudges on error/unchanged
- Add `sift.store()` for explicit content storage with auto-nudge
- Make all `sift.*` functions ctx-first for consistency
- Add multi-pattern plugin support (`pattern` accepts `string | string[]`)
- Ship openspec.lua and rtk.lua as optional plugins
- On-error output storage with auto-nudge (replace always-save)
- Remove specific CommandKind variants (dead code), keep general parsing
- Handle `cd <dir> && <command>` in dispatch — peel cd, chdir, dispatch rest through plugins

**Non-Goals:**
- Changing the core dispatch algorithm (longest-prefix + priority stays)
- Adding new dependencies (rtk is called as external binary)
- Windows support

## Decisions

### D1 — Core vs optional split: embedded vs filesystem

Core plugins (bash, command, reset) stay in `sift/plugins/` loaded via `include_str!()`. Optional plugins live in top-level `plugins/` directory, loaded at startup via filesystem scan. The scan order is: `plugins/` (shipped), `~/.config/sift/plugins/` (user overrides), `$SIFT_PLUGINS` (env). Later scans override earlier ones at the same priority.

**Rationale**: Core plugins are essential for sift's operation (fallback, bypass, reset). Optional plugins are transformations that users can customize or disable by removing the file.

### D2 — ctx-first API: all functions

Every `sift.*` function takes `ctx` as first argument. Functions that don't currently need it (hash, json, toon, etc.) accept it for future-proofing and consistency. The `ctx` table is always available inside plugin `execute()`.

**Rationale**: Consistent API surface. Future capabilities (session-scoped caching, per-command tracking) can be added without breaking signatures.

### D3 — sift.json.shortest(): token-aware JSON optimization

Takes raw JSON, tries each format in `formats` table, measures token cost of each output + nudge overhead, picks the shortest, stores raw original on disk, auto-nudges if raw wasn't selected. Only operates on valid JSON — non-JSON returns raw unchanged.

Formats tried:
- `raw` (always baseline, no nudge)
- `json { max_string_len, max_array_items, max_depth, max_keys }` — compacted JSON
- `toon {}` — TOON format

**Rationale**: The primary goal is token minimization. By measuring actual token cost including nudge, the function guarantees the shortest representation is always selected.

### D4 — Nudge system: explicit + automatic

`sift.log.nudge(ctx, msg)` accumulates messages during plugin execution. At the end of dispatch, all nudges are appended to the plugin's output as `[sift] <msg>` lines. Automatic nudges are triggered by:
- `sift.exec()` non-zero exit → `[sift] use 'command cat <path>' for raw output`
- Plugin returns `status = "unchanged"` → `[sift] use 'command cat <path>' for unfiltered content`
- `sift.json.shortest()` selects non-raw format → `[sift] use 'command cat <path>' for raw original`
- `sift.store()` → `[sift] use 'command cat <path>' for <slug>`

**Rationale**: Nudges tell the agent exactly what command to run to get original/unfiltered content. The `command cat` bypass ensures the agent gets raw binary output.

### D5 — Multi-pattern: string | string[]

Plugin `pattern` field accepts either a string or an array of strings. The `find_plugin()` candidate builder checks each candidate against all patterns in the array. If any matches, the plugin is selected.

**Rationale**: A single rtk plugin can match `docker`, `podman`, `kubectl`, `oc`, etc. without registering multiple plugin instances.

### D6 — On-error output storage

`sift.exec()` saves raw output to `/tmp/sift/<session>/<ts>_<cmd_count>_<slug>.log` only when `exit_code != 0`. On success, raw output is discarded. The nudge includes the full path.

**Rationale**: Saves disk space and I/O. Errors are the important case for debugging. Success output can be re-generated.

### D7 — Optional plugin patterns

- `git_status.lua`: pattern changes from `"git"` to `"git status"` — the longest-prefix dispatch ensures it beats a `"git"`-patterned rtk plugin for `git status` commands, while `git diff` falls through to rtk.
- `openspec.lua`: pattern `"openspec"` — injects `--json` if missing, converts output to toon via `sift.json.shortest()`.
- `rtk.lua`: pattern `["docker", "podman", "kubectl", "oc", "gh", "glab", "curl", "wget", "npm", "pnpm", "pip", "uv"]` — delegates matching commands to rtk binary.

### D8 — Classifier simplification: remove CommandKind

The `CommandKind` enum (SimpleFileRead, FileRead, CargoTest, CargoBuild, GitStatus, GitDiff, Interactive, Unknown) and its `classify_command()` function are dead code. No plugin uses them. The dispatch system doesn't reference CommandKind. `sift.classify()` returns a simplified `{name, args, is_piped, is_compound}` without the `kind` field. The brush-parser parsing infrastructure stays — it's still used internally by the dispatch and useful for plugins that want to introspect commands.

**Rationale**: 50 lines of dead code maintained for nothing. The specific classifications (CargoTest, GitDiff) were an artifact of an earlier architecture where plugins were selected by classification. Now plugins are selected by pattern matching. The classification is unused.

### D9 — `cd <dir> && <command>` handling in dispatch

When the dispatch function receives a compound command matching `cd <dir> && <rest>`, it SHALL:
1. Extract `<dir>` and `<rest>` from the compound
2. Change sift's working directory to `<dir>` via `std::env::set_current_dir()`
3. Dispatch `<rest>` against plugins normally

This ensures `cd /x && docker ps` dispatches `docker ps` through rtk.lua instead of running the whole string raw through bash.lua.

The pattern is detected using the existing classifier infrastructure: if `is_compound` is true and the first command is `cd` with exactly one argument, peel it. More complex cases (`cd /x && cd /y && cmd`, `cd /x; cmd`) are handled recursively: each `cd` is peeled, the remainder is re-dispatched.

**Rationale**: Users commonly change directory and run a command in one line. The agent should get plugin-optimized output for the actual command, not raw bash output for the entire compound.

**State variable**: `cd_handling ∈ {peel_and_dispatch}`

### D10 — Pipeline optimization: last-plugin dispatch

When a command contains pipes (`|`), the dispatch SHALL parse the pipeline and check if the LAST command matches a plugin. If it does, the dispatch SHALL:
1. Execute all preceding commands in bash, capturing their combined stdout
2. Pass the captured stdout as `stdin` to the last command's plugin
3. The plugin processes, caches, and returns the output as normal

If the last command does NOT match any plugin, the entire pipeline SHALL run in bash (current behavior).

This enables optimization for patterns like `echo abc | cat` where cat.lua caches the piped content, or `kubectl get pods -o json | sift.json.shortest()` where the JSON optimizer compacts the piped output.

**Rationale**: The last command in a pipeline is the final output producer — the only one the agent sees. By dispatching it through plugins, we get caching, compression, and transformation on piped content. The preceding commands are pure data generators that run normally in bash.

**State variable**: `pipeline_dispatch ∈ {last_cmd_plugin, bash_full_pipeline}`

## Risks / Trade-offs

- **[ctx-first verbosity]** → All function calls are slightly longer. Acceptable for API consistency and future-proofing.
- **[rtk binary dependency]** → rtk must be installed separately. If not found, `sift.exec("rtk ...")` returns non-zero and falls through to bash.lua default.
- **[TOON overhead on small JSON]** → For small JSON (<50 tokens), the nudge overhead may exceed savings. `sift.json.shortest()` handles this automatically by comparing costs — raw wins when savings are too small.
- **[Plugin relocation breaks user configs]** → Users with custom cat.lua or git_status.lua in `~/.config/sift/plugins/` will have their versions loaded instead of the shipped ones. The `plugins/` directory is scanned before user config, so user overrides work naturally.
- **[cd && chdir side effect]** → Changing sift's cwd on `cd /x && cmd` affects all subsequent commands. This matches shell behavior and is the user's intent. If the cd fails, the command is dispatched as-is (bash will report the error).
