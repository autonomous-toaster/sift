## Why

When sift-read reads a file whose content hasn't changed but the range is new (e.g., read 1-4 then 1-5), the diff path triggers: `sift.diff(old, new)` returns an empty string (no changes), the usefulness gate `#diff < #content * 0.9` passes (0 < anything), and the empty diff is returned instead of the actual content. The agent sees nothing.

## What Changes

- **FIX**: Add `#diff > 0` check to the usefulness gate in sift-read.lua.
- **NEW**: Regression tests for: empty diff return, range boundary, cross-range unchanged detection.

## Capabilities

### Modified Capabilities
- `sift-read-plugin`: Diff usefulness gate now requires non-empty diff.

## Impact

- **plugins/sift-read.lua**: One-line change: `if #diff > 0 and #diff < #content * 0.9 then`
- **sift-core/src/lua/mod.rs**: Add regression tests.
