# Tasks

## 1. Read tool: custom execute function

- [x] Replace `createReadTool` with custom tool definition in `integrations/pi/sift.ts`
- [x] `execute` calls `siftExec("sift-read " + shQuote(path))`, returns output directly
- [x] Handle image files by reading directly
- [x] Remove `createReadTool` import

## 2. Output duplication: bash plugin stops returning `output`

- [x] Edit `sift/plugins/bash.lua`: remove `output` field from return value
- [x] Verify `dispatch()` in `api.rs` handles missing `output` (no code change needed)

## 3. Redirect handling in `dispatch_full`

- [x] Add `< file` redirect parsing in `sift-core/src/lua/api.rs` `dispatch_full()`
- [x] Add `> file` and `>> file` redirect parsing
- [x] Complex redirects fall through to shell

## 4. spawnHook: `JSON.stringify`

- [x] Replace `shQuote(command)` with `JSON.stringify(command)` in spawnHook
- [x] Remove `shQuote` function if no longer used elsewhere

## 5. System prompt nudge

- [x] Add `before_agent_start` handler in `integrations/pi/sift.ts`
- [x] Append nudge text to `event.systemPrompt`
- [x] Nudge text is stable (same every turn) to preserve prompt caching
