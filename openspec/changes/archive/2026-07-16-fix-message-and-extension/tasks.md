# Tasks

## 1. Fix sift-read message format

- [ ] Edit `plugins/sift-read.lua`: when `range_start == range_end`, emit "line X" instead of "lines X-X"

## 2. Fix pi extension

- [ ] Add `shQuote()` function to `integrations/pi/sift.ts`
- [ ] Add module-level `currentSessionId` variable
- [ ] Add `session_start` event handler to get real session ID
- [ ] Fix `spawnHook` to use `currentSessionId`
- [ ] Replace read tool with `createReadTool` + custom `ReadOperations`
- [ ] Fix reset cache handlers to use `currentSessionId`
