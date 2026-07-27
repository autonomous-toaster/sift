## 1. Add nudge to cat.lua on markdown compression

- [x] 1.1 In `plugins/cat.lua`, after the mdmin compression block, add `sift.nudge(ctx, "raw: 'command cat " .. path .. "'")` to inform the agent the content was compressed
- [x] 1.2 Verify: reading a `.md` file through cat produces a `[sift] raw: 'command cat ...'` nudge

## 2. Add nudge to sift-read.lua on content compression

- [x] 2.1 In `plugins/sift-read.lua`, after content compression (xberg or mdmin), add `sift.nudge(ctx, "raw: 'command cat " .. path .. "'")` to inform the agent the content was compressed
- [x] 2.2 Verify: reading a `.md` file through sift-read produces a `[sift] raw: 'command cat ...'` nudge
