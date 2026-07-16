## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `sift.cache.add_range(ctx, hash, start, end)` with merge logic |
| T1.2 | Add `sift.cache.has_range(ctx, hash, start, end)` with union containment |
| T2.1 | Update sift-read.lua to use range-aware cache |

## ADDED Requirements

### Requirement: add_range merges overlapping ranges

T1.1 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Adjacent ranges merged
- **WHEN** T1.1 adds range `[1,6]` then `[7,10]`
- **THEN** T1.1 SHALL store a single range `[1,10]`.

#### Scenario: Overlapping ranges merged
- **WHEN** T1.1 adds range `[1,5]` then `[3,10]`
- **THEN** T1.1 SHALL store a single range `[1,10]`.

### Requirement: has_range checks union containment

T1.2 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Range fully contained
- **WHEN** T1.2 checks `[3,5]` and cached ranges include `[1,10]`
- **THEN** T1.2 SHALL return true.

#### Scenario: Range not fully contained
- **WHEN** T1.2 checks `[1,10]` and cached ranges are `[1,6]` and `[8,10]`
- **THEN** T1.2 SHALL return false (line 7 was never read).

### Requirement: sift-read uses range cache

T2.1 SHALL complete AFTER T1.1 AND T1.2 SHALL complete.

#### Scenario: Range read caches range
- **WHEN** T2.1 reads lines `5-10` of a file
- **THEN** T2.1 SHALL add range `[5,10]` to the cache marker.

#### Scenario: Sub-range returns unchanged
- **WHEN** T2.1 reads lines `6-8` and range `[5,10]` is cached
- **THEN** T2.1 SHALL return "unchanged".
