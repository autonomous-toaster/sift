## Phase 1: append_prompt

- [x] 1.1 In `sift-core/src/lua/api.rs`, add `append_prompt: Option<String>` to `PluginEntry` struct and read it in `load_plugin_from_str()`
- [x] 1.2 In `sift/src/main.rs`, add `--append-prompt` CLI flag that initializes Lua, loads plugins, collects prompts, outputs them, and exits
- [x] 1.3 Add `append_prompt` to built-in plugins:
  - `plugins/cat.lua`: "Markdown files may be compressed via mdmin. Follow [nudge] raw: hints to get the original content."
  - `plugins/curl.lua`: "HTTP responses are transformed (JSON compressed, PDF→markdown). Follow [nudge] raw: hints to access the saved response without re-issuing the request."
  - `plugins/rtk.lua`: "Output from git, docker, ls and other commands is compressed by rtk. Follow [nudge] command: hints to get raw output."
  - `plugins/sift-read.lua`: "File contents may be compressed or extracted. Use sift-read --raw <path> for original content."
- [x] 1.4 In `integrations/pi/sift.ts`, update `before_agent_start` to call `sift --append-prompt` and append output to system prompt

## Phase 2: Rename [sift] → [nudge]

- [x] 2.1 Rename `[sift]` → `[nudge]` in all plugin Lua files: cat.lua, head.lua, sed.lua, tail.lua, sift-read.lua, reset.lua
- [x] 2.2 Rename `[sift]` → `[nudge]` in Rust code: api.rs (nudge collection, unchanged burst message). Keep `[sift]` for log messages in api_reg_cache.rs.
- [x] 2.3 Update tests: tests.rs, cli.rs to expect `[nudge]` prefix

## Phase 3: Add [nudge] command: hints

- [x] 3.1 In `plugins/rtk.lua`, add `sift.nudge(ctx, "command: '" .. ctx.original_cmd .. "'")` on successful execution
- [x] 3.2 In `plugins/jq.lua`, add `sift.nudge(ctx, "command: '" .. ctx.original_cmd .. "'")` on successful execution
- [x] 3.3 Git commands are handled by rtk plugin (already has command nudge from 3.1)
