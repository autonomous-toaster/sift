# Design

## How gain tracking works

```
Plugin returns { status, output, raw_bytes? }
                        │
                        ▼
  dispatch computes:
    raw_bytes      = plugin's raw_bytes
                     or falls back to filtered_bytes (0% reduction)
    filtered_bytes = final_output.len()  ← includes nudge text
    reduction_bps  = (raw - filtered) * 10000 / raw
                        │
                        ▼
  record_conversation(raw_bytes, filtered_bytes, plugin_name, output_format)
                        │
                        ▼
  gain report aggregates by plugin_name
```

## Changes per plugin

### head.lua, tail.lua, sed.lua

These plugins read a file and return a subset of lines. The `raw_bytes` should be the full file size (`stat.size`), not the sliced output size.

Current flow:
```
sift.fs.read(path) → content
sift.str.slice_text(content, start, end) → sliced
return { output = sliced }  ← no raw_bytes
```

Fixed flow:
```
sift.fs.stat(path) → stat        ← NEW: get file size
sift.fs.read(path) → content
sift.str.slice_text(content, start, end) → sliced
return { output = sliced, raw_bytes = stat.size }
```

For "unchanged" responses, add `raw_bytes = stat.size` so the nudge size is compared against the full file size.

### openspec.lua

This plugin runs an openspec command, captures JSON output, and compresses it via `sift.json.shortest()`. The `raw_bytes` should be the raw JSON output size (before compression).

Current flow:
```
sift.exec("openspec ...") → output
sift.json.shortest(ctx, output, formats) → optimized
return { output = optimized }  ← no raw_bytes
```

Fixed flow:
```
sift.exec("openspec ...") → output
sift.json.shortest(ctx, output, formats) → optimized
return { output = optimized, raw_bytes = #output }  ← raw JSON size
```

## Nudge accounting

Nudges from `sift.store()`, `sift.json.shortest()`, and `sift.nudge()` are collected by `collect_nudges()` and appended to `final_output` in the dispatch code. Since `filtered_bytes = final_output.len()`, the nudge overhead is already included in the reduction calculation. No changes needed for nudge accounting.
