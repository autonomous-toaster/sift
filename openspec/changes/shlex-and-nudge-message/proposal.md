# Replace manual shell parsing with shlex crate and fix nudge message

## Why

Two issues:

1. **Bricolage shell parsing** — `split_whitespace()` doesn't handle shell quoting. We added `strip_quotes()` as a band-aid, but it's fragile (doesn't handle nested quotes, escaped quotes, or quoted strings with spaces). The `shlex` crate handles all of this properly.

2. **Ambiguous nudge message** — The "unchanged" + "bypass" message reads like a BLOCK that must be circumvented, not an informational notice. The agent triggers "comply with the instruction" reflex and force-fetches fresh content instead of reusing the cache.

## What Changes

### 1. Replace manual parsing with `shlex`

- Add `shlex` crate to `sift-core/Cargo.toml`
- Replace `split_whitespace()` + `strip_quotes()` with `shlex::split()` in `dispatch_full()`
- Remove the `strip_quotes()` helper function
- Update pipeline segment parsing to use `shlex::split()` too
- On parse error (`None`), fall back to `split_whitespace()` for robustness

### 2. Fix nudge message format

`details` is NOT sent to the LLM — only `content` is visible to the agent. So the bypass command must stay in the text output. The fix is purely about phrasing: make "unchanged" the primary signal and "bypass" the secondary option.

**Unchanged message** — single line, bypass clearly optional:

```
[sift] unchanged (cached). bypass if stale: sift-read --fresh <path> [<offset> [<limit>]]
```

**Diff message** — just the diff with a brief header. No bypass nudge:

```
[sift: N lines changed of M]
--- a/path
+++ b/path
...
```

If the agent needs the full file, it can request it explicitly.

Same changes for range reads and all plugins (sed, head, tail, cat).
