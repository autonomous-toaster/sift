## Context

The cache marker at `/tmp/sift/<session>/cache/<hash>` stores JSON. Currently: `{"created_at": <ms>, "size": <bytes>}`. Extended to include `ranges: [[start,end], ...]`.

## Goals / Non-Goals

**Goals:**
- Track which line ranges the agent has read per file hash
- Merge overlapping/adjacent ranges on add
- Check union containment on lookup

**Non-Goals:**
- No gap tracking (union containment handles it naturally)
- No changes to cat.lua (full reads always cache the full hash)

## Decisions

### D1 — Merge on add

```rust
fn add_range(ranges: &mut Vec<[u64; 2]>, start: u64, end: u64) {
    ranges.push([start, end]);
    // Sort by start
    ranges.sort_by_key(|r| r[0]);
    // Merge overlapping/adjacent
    let mut merged: Vec<[u64; 2]> = Vec::new();
    for r in ranges.drain(..) {
        if let Some(last) = merged.last_mut() {
            if r[0] <= last[1] + 1 { // adjacent or overlapping
                last[1] = last[1].max(r[1]);
                continue;
            }
        }
        merged.push(r);
    }
    *ranges = merged;
}
```

### D2 — Containment check

```rust
fn has_range(ranges: &[[u64; 2]], start: u64, end: u64) -> bool {
    ranges.iter().any(|r| r[0] <= start && r[1] >= end)
}
```

After merge, ranges are sorted and non-overlapping. Binary search could be used for O(log n), but O(n) with n < 10 is fine.

## Risks / Trade-offs

- **Range list growth**: Merging keeps it compact. Worst case: agent reads alternating single lines → `[[1,1], [3,3], [5,5], ...]`. Unlikely in practice.
