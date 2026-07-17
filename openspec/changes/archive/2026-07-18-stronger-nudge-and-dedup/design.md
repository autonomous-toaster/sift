# Design

## 1. Stronger system prompt nudge

**File**: `integrations/pi/sift.ts`

Replace the `before_agent_start` handler text:

```typescript
pi.on("before_agent_start", async (event) => {
  return {
    systemPrompt:
      event.systemPrompt +
      '\n\n[sift] caches file reads. When you see "[sift] ... unchanged (cached)", ' +
      'the content is already in this conversation — say "same as before" and move on. ' +
      "Do NOT re-read or bypass the cache unless you have a specific reason to believe " +
      "the file changed on disk.",
  };
});
```

## 2. Keep bypass command but de-emphasize it

**Files**: `plugins/sift-read.lua`, `plugins/cat.lua`, `plugins/sed.lua`, `plugins/head.lua`, `plugins/tail.lua`

Change from single-line format:
```
[sift] Justfile unchanged (cached). bypass if stale: command cat /path/Justfile
```

To two-line format where the second line is indented and parenthetical (reads as secondary info):
```
[sift] Justfile unchanged (cached)
      (bypass if stale: command cat /path/Justfile)
```

The bypass command is preserved (needed for per-command bypass like `command cat`, `command sed`, etc.) but visually de-emphasized.

## 3. Dedup protection with time-based burst detection

**File**: `sift-core/src/lua/api.rs` — `dispatch()` function

Track recent unchanged responses with timestamps. Use a `Mutex<Vec<(String, u128)>>` field in `SiftLua`:

```rust
// In dispatch(), after handling unchanged status:
if status == "unchanged" {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let key = format!("{}:{}", cmd, msg);
    let mut recent = self.recent_unchanged.lock().unwrap();
    // Prune entries older than 10 seconds
    recent.retain(|(_, ts)| now.saturating_sub(*ts) < 10_000);
    recent.push((key.clone(), now));
    // Keep sliding window of last 10
    if recent.len() > 10 { recent.remove(0); }
    // Count occurrences of this key in the window
    let count = recent.iter().filter(|(k, _)| k == &key).count();
    if count >= 3 {
        msg = format!("{}\n[sift] (this will keep returning the same result until the file changes on disk)", msg);
    }
}
```

Time-based burst detection: 3+ identical unchanged responses within 10 seconds. Prevents false positives from spaced-out reads across different turns.
