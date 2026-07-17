# Tasks

## 1. Stronger system prompt nudge

- [x] Update `before_agent_start` handler in `integrations/pi/sift.ts` with stronger nudge text
- [x] Text tells agent to say "same as before" and NOT to re-read

## 2. Remove "bypass" from unchanged message

- [x] Update `plugins/sift-read.lua` — two-line format, bypass de-emphasized on second line
- [x] Update `plugins/cat.lua` — two-line format, bypass de-emphasized on second line
- [x] Update `plugins/sed.lua` — two-line format, bypass de-emphasized on second line
- [x] Update `plugins/head.lua` — two-line format, bypass de-emphasized on second line
- [x] Update `plugins/tail.lua` — two-line format, bypass de-emphasized on second line

## 3. Dedup protection

- [x] Add `Mutex<Vec<(String, u128)>>` field to `SiftLua` struct
- [x] Track recent command+status pairs with timestamps in `dispatch()`
- [x] Prune entries older than 10s before each check
- [x] Append stronger hint on 3+ repeats within 10s window
- [x] Run tests to verify
