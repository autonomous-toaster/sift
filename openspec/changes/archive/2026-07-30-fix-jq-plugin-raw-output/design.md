## Context

The jq plugin's `-r` (raw output) path decodes the JSON result and iterates with `ipairs()` to extract values as strings. `ipairs()` only iterates over array-like Lua tables (consecutive integer keys from 1). When `sift.jq.query()` returns a single value (e.g., a job ID), `sift.json.decode()` returns a Lua scalar — `ipairs()` produces nothing, and the output is empty.

## Goals / Non-Goals

**Goals:**
- Handle single-value results in the `-r` path (e.g., `jq -r '.id'`)
- Handle empty results gracefully (no matches)
- Preserve existing behavior for array results

**Non-Goals:**
- Changing the non-`-r` path (it handles scalars correctly via `sift.json.shortest()`)
- Adding new jq features or flags

## Decisions

### Decision 1: Type-check before iterating

Use `type(decoded) == "table"` to distinguish arrays from scalars:

```lua
if type(decoded) == "table" then
    for _, v in ipairs(decoded) do
        lines[#lines + 1] = tostring(v)
    end
else
    lines[#lines + 1] = tostring(decoded)
end
```

This is minimal, correct, and preserves existing behavior for arrays.

### Decision 2: Empty results fall through to passthrough

If `sift.jq.query()` returns an empty result (no matches), `sift.json.decode()` may fail (empty string is not valid JSON). The existing `pcall` guard returns passthrough, which runs the real `jq` — correct behavior.

## Risks / Trade-offs

- **Minimal change**: 3 lines added, 0 removed. Low risk.
- **Edge case**: `null` result from jq. `sift.json.decode("null")` returns Lua `nil`. `type(nil)` is `"nil"`, not `"table"`, so it falls to the `else` branch: `tostring(nil)` returns `"nil"`. This is acceptable — `jq -r` with a null-producing filter would output "null" in real jq too.
