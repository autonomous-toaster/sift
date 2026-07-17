# Design

## 1. Replace manual parsing with `shlex`

**File**: `sift-core/Cargo.toml` — add `shlex = "2"` dependency

**File**: `sift-core/src/lua/api.rs`

Replace `split_whitespace()` + `strip_quotes()` with `shlex::split()`:

```rust
// Before
let parts: Vec<&str> = full_cmd.split_whitespace().collect();
let name = parts[0];
let args: Vec<String> = parts[1..].iter().map(|s| strip_quotes(s)).collect();

// After
let parts = shlex::split(full_cmd).unwrap_or_else(|| {
    full_cmd.split_whitespace().map(String::from).collect()
});
if parts.is_empty() {
    return Ok((String::new(), 0, String::new()));
}
let name = &parts[0];
let args: Vec<String> = parts[1..].to_vec();
```

Same change for pipeline segment parsing. Remove the `strip_quotes()` function entirely.

`shlex::split()` handles:
- Single quotes: `'hello world'` → `["hello world"]`
- Double quotes: `"hello world"` → `["hello world"]`
- Escaped characters: `hello\ world` → `["hello world"]`
- Mixed quoting: `"file with spaces.txt"` → `["file with spaces.txt"]`

On parse error (`None`), fall back to `split_whitespace()` for robustness.

## 2. Fix nudge message format

`details` is NOT sent to the LLM — only `content` is visible. The bypass command must stay in text output. Fix is purely about phrasing.

### Unchanged message

Single line, bypass clearly optional:

```
[sift] unchanged (cached). bypass if stale: sift-read --fresh <path> [<offset> [<limit>]]
```

For range reads:
```
[sift] lines X-Y unchanged (cached). bypass if stale: sift-read --fresh <path> <offset> <limit>
```

### Diff message

Just the diff with a brief header. No bypass nudge:

```
[sift: N lines changed of M]
--- a/path
+++ b/path
...
```

### Affected plugins

- `plugins/sift-read.lua` — full read and range read messages
- `plugins/cat.lua` — unchanged message
- `plugins/sed.lua` — unchanged message
- `plugins/head.lua` — unchanged message
- `plugins/tail.lua` — unchanged message

### Diff message

Just the diff with a brief header. No bypass nudge.

```
[sift: N lines changed of M]
--- a/path
+++ b/path
@@ -1,5 +1,5 @@
...
```

If the agent needs the full file, it can request it explicitly.

### Bypass commands per plugin

The bypass command preserves the original arguments (path, offset, limit) so the agent gets the same slice fresh:

- sift-read: `sift-read --fresh <path> [<offset> [<limit>]]`
- cat: `command cat <path>`
- sed: `command sed -n '<start>,<end>p' <path>`
- head: `command head -n <count> <path>`
- tail: `command tail -n <count> <path>`
