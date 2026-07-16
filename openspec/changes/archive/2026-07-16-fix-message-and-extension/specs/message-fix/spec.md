# sift-read message format fix

## MODIFIED

### sift-read unchanged message

- When `range_start == range_end`, the "unchanged" message says "line X" instead of "lines X-X"
- When `range_start < range_end`, the message stays "lines X-Y"
- No other behavior changes

## Verification

- `sift-read file 10 1` → `[sift] file line 10 unchanged`
- `sift-read file 10 11` → `[sift] file lines 10-20 unchanged`
