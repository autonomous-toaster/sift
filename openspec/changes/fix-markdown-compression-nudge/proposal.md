## Why

When a coding agent reads a `.md` file through sift, the cat and sift-read plugins silently compress the markdown content via mdmin. The agent sees compressed markdown, constructs edit operations based on it, and the edits fail because the actual file on disk has uncompressed content.

The curl plugin handles this correctly: it stores the raw response and nudges the agent with `[sift] raw: 'command cat ...'`. The cat and sift-read plugins have no such nudge — the agent doesn't know the content was compressed.

## What Changes

Add a nudge message when cat.lua and sift-read.lua compress content, telling the agent how to get the raw (uncompressed) version using `command cat`.

## Capabilities

### New Capabilities
- `compression-nudge`: When file content is compressed before being returned to the agent, a nudge is emitted telling the agent how to access the raw content.

### Modified Capabilities
*(No existing capability specs are changing.)*

## Impact

- `plugins/cat.lua`: Add `sift.nudge()` after markdown compression
- `plugins/sift-read.lua`: Add `sift.nudge()` after content compression
