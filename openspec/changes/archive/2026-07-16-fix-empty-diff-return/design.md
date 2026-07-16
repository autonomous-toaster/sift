## Context

The diff usefulness gate `#diff < #content * 0.9` passes when diff is empty (0 < anything). This causes sift-read to return an empty string when the file hasn't changed but the range is new.

## Fix

```lua
-- Before
if #diff < #content * 0.9 then

-- After
if #diff > 0 and #diff < #content * 0.9 then
```

## Tests

Three regression tests in `sift-core/src/lua/mod.rs`:
1. `test_sift_read_empty_diff` — read 1-4 then 1-5, verify content returned
2. `test_sift_read_unchanged_range` — read 1-4 twice, verify "unchanged"
3. `test_sift_read_sub_range` — read 1-10 then 3-5, verify content returned
