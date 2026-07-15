## 1. Ctx-first API migration

- [x] 1.1 Add `ctx` as first arg to all `sift.*` register_* functions in `lua.rs`: exec, hash.*, fs.*, json.*, toon.*, jq.*, env.*, classify, log, token_count.
- [x] 1.2 Update all built-in plugins (bash.lua, command.lua, reset.lua) to pass ctx as first arg.
- [x] 1.3 Update all optional plugins (cat.lua, git_status.lua) to pass ctx as first arg.
- [x] 1.4 Update all tests in lua.rs to use new ctx-first signatures.

## 2. Nudge system

- [x] 2.1 Add nudge accumulator (Vec<String>) to SiftLua runtime.
- [x] 2.2 Implement `sift.log.nudge(ctx, msg)` — pushes to accumulator.
- [x] 2.3 At end of dispatch(), append accumulated nudges to plugin output as `[sift] <msg>` lines.
- [x] 2.4 Automatic nudge on `sift.exec()` non-zero exit: store raw output, emit `[sift] use 'command cat <path>' for raw output`.
- [x] 2.5 Automatic nudge on plugin returning `status = "unchanged"`: emit bypass hint.

## 3. Store primitive

- [x] 3.1 Implement `sift.store(ctx, content, slug)` — write to `/tmp/sift/<session>/<ts>_<count>_<slug>`, return path, emit nudge.

## 4. sift.json.shortest()

- [x] 4.1 Implement `sift.json.shortest(ctx, raw, formats)` in Rust: token cost comparison, format selection.
- [x] 4.2 Implement compacted JSON output: truncate long strings, summarize large arrays, limit depth/keys.
- [x] 4.3 Wire auto-nudge + raw storage when non-raw format wins.
- [x] 4.4 Add tests for token cost comparison, format selection, edge cases (non-JSON, empty, tiny JSON).

## 5. Plugin optionalization

- [x] 5.1 Move cat.lua and git_status.lua from `sift/plugins/` to top-level `plugins/`.
- [x] 5.2 Remove cat.lua and git_status.lua from `load_builtin_plugins()` in main.rs.
- [x] 5.3 Add `plugins/` directory scan to `load_user_plugins()` or equivalent in main.rs.
- [x] 5.4 Implement multi-pattern support: `pattern` accepts `string | string[]` in find_plugin().
- [x] 5.5 Change git_status.lua pattern from `"git"` to `"git status"`.

## 6. Optional plugins

- [x] 6.1 Write `plugins/openspec.lua`: inject --json, convert via sift.json.shortest().
- [x] 6.2 Write `plugins/rtk.lua`: delegate docker/podman/kubectl/oc/gh/glab/curl/wget/npm/pnpm/pip/uv to rtk.

## 7. Classifier simplification and cd dispatch

- [x] 7.1 Remove `CommandKind` enum and `classify_command()` from classifier.rs.
- [x] 7.2 Simplify `Classification` struct to `{name, args, is_piped, is_compound}` without `kind`.
- [x] 7.3 Update `sift.classify()` Lua binding to return simplified struct.
- [x] 7.4 Update classifier tests (remove kind-specific assertions).
- [x] 7.5 Implement `cd <dir> && <command>` peel-and-dispatch in the dispatch function.
- [x] 7.6 Handle recursive cd chains: `cd /x && cd /y && cmd`.
- [x] 7.7 Handle `pushd <dir> && <command>` and `popd`.
- [x] 7.8 Handle `cd <dir> ; <command>` (semicolon separator).

## 8. Pipeline optimization

- [x] 8.1 Parse pipeline structure from classified command in dispatch.
- [x] 8.2 If last command matches a plugin, run preceding commands in bash, pipe to plugin.
- [x] 8.3 If last command does not match a plugin, fall through to bash for whole pipeline.
- [x] 8.4 Update cat.lua to handle piped stdin (cache by hash, return unchanged on repeat).
- [x] 8.5 Update dispatch in agent_mode and repl_mode to use classifier for pipeline detection.

## 9. Migration and cleanup

- [x] 9.1 Update docs/examples/ to reflect new plugin locations and ctx-first signatures.
- [x] 9.2 Verify `just ci` passes with all changes.
