## Why

The current sift embeds all plugins in the binary, conflating core infrastructure (bash fallback, bypass, reset) with optional transformations (cat, git status). JSON handling is limited to standard serialize/deserialize — no smart token minimization. There's no way to delegate commands to external tools like rtk, no error-output recovery mechanism, and no unified nudge system to tell the agent where to find original content. The API lacks consistency (some functions take ctx, most don't).

## What Changes

- **BREAKING**: Core plugins (bash.lua, command.lua, reset.lua) stay embedded. All other plugins move to top-level `plugins/` directory, loaded from filesystem.
- **BREAKING**: Every `sift.*` function takes `ctx` as first argument for API consistency and future-proofing.
- **BREAKING**: `sift.classify(cmd)` simplified — `CommandKind` enum removed (CargoTest, GitStatus, etc. were dead code). Returns `{name, args, is_piped, is_compound}` only.
- **NEW**: `sift.json.shortest(ctx, raw_json, formats)` — tries multiple JSON representations (raw, compacted, TOON), measures token cost including nudge overhead, picks the shortest, stores raw original, auto-nudges.
- **NEW**: `sift.log.nudge(ctx, msg)` — explicit nudge primitive. Accumulates messages during plugin execution, appended to output as `[sift] <msg>` lines.
- **NEW**: `sift.store(ctx, content, slug)` — store content to disk, return path, auto-nudge.
- **NEW**: Automatic nudge on `sift.exec()` non-zero exit — stores raw output, emits `[sift] use 'command cat <path>' for raw output`.
- **NEW**: Automatic nudge on plugin returning `status = "unchanged"` — emits bypass hint with path.
- **NEW**: `plugins/` directory at project root — shipped optional plugins (cat.lua, git_status.lua, openspec.lua, rtk.lua) loaded from filesystem, overrideable by user config.
- **NEW**: Multi-pattern plugin support — `pattern` accepts `string | string[]`.
- **NEW**: `cd <dir> && <command>` dispatch handling — peel off `cd` prefix, chdir, dispatch `<command>` against plugins instead of running raw through bash.
- **NEW**: Pipeline optimization — if the LAST command in a pipe matches a plugin, run preceding commands in bash, pipe output to the plugin for processing/caching.
- **NEW**: `openspec.lua` plugin — injects `--json` flag, converts output to toon.
- **NEW**: `rtk.lua` plugin — delegates matching commands (docker, podman, kubectl, etc.) to rtk.
- **MODIFIED**: `git_status.lua` pattern changes from `"git"` to `"git status"` for correct longest-prefix dispatch with rtk plugin.
- **REMOVED**: `sift/output-storage` — always-save behavior replaced by on-error save with auto-nudge.

## Capabilities

### New Capabilities
- `json-shortest`: `sift.json.shortest()` — token-aware JSON representation selector (raw vs compacted vs toon)
- `nudge-system`: `sift.log.nudge()` + automatic nudges on error/unchanged/bypass
- `store-primitive`: `sift.store()` for explicit content storage with auto-nudge
- `plugin-optionalization`: Split core/optional plugins, top-level `plugins/` directory, multi-pattern support
- `openspec-plugin`: Built-in openspec.lua with --json injection and toon conversion
- `rtk-plugin`: Built-in rtk.lua for delegating commands to rtk
- `ctx-first-api`: Every sift.* function takes ctx as first argument
- `classifier-simplify`: Strip CommandKind, keep general parsing, no dead code
- `pipeline-optimization`: Dispatch last command through plugins when piped

### Modified Capabilities
- `sift-api`: All functions gain ctx-first signature. `sift.exec()` gets auto-nudge on error. `sift.log` gains `nudge` level.
- `plugin-system`: `pattern` field accepts string or string[]. Plugin loading scans `plugins/` directory. `cd <dir> && <command>` pattern handled in dispatch.
- `output-storage`: Changed from always-save to on-error-save with auto-nudge.

## Impact

- **New directory**: `plugins/` at project root with cat.lua, git_status.lua, openspec.lua, rtk.lua
- **Moved files**: cat.lua, git_status.lua from `sift/plugins/` to `plugins/`
- **Core plugins**: bash.lua, command.lua, reset.lua stay embedded in `sift/plugins/`
- **Removed code**: `CommandKind` enum and `classify_command()` function — 50 lines of dead code
- **API surface**: every sift.* function gains ctx first arg — all existing plugins must update. `sift.classify()` returns simplified struct without `kind`.
- **New behavior**: `cd /x && docker ps` dispatches `docker ps` through plugins (rtk.lua), not raw bash
- **New behavior**: `echo abc | cat` pipes through cat.lua (caching), not raw bash
- **New dependencies**: none (rtk is called as external binary, not linked)
- **Breaking changes**: ctx-first signatures, plugin relocation, output-storage behavior, classifier simplification
