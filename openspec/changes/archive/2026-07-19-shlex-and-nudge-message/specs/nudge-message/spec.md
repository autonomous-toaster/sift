# Fix nudge message format

## MODIFIED

### Unchanged message format

Single line, bypass clearly optional:
```
[sift] unchanged (cached). bypass if stale: sift-read --fresh <path> [<offset> [<limit>]]
```

For range reads:
```
[sift] lines X-Y unchanged (cached). bypass if stale: sift-read --fresh <path> <offset> <limit>
```

### Diff message format

Just the diff with a brief header `[sift: N lines changed of M]`. No bypass nudge.

### Affected plugins

- `plugins/sift-read.lua` — full read and range read messages
- `plugins/cat.lua` — unchanged message
- `plugins/sed.lua` — unchanged message
- `plugins/head.lua` — unchanged message
- `plugins/tail.lua` — unchanged message

## Verification

- Agent sees "unchanged (cached)" as primary signal → reuses cached content
- Agent sees "bypass if stale" as secondary → uses it only when needed
- No more force-fetching fresh content on re-read requests
