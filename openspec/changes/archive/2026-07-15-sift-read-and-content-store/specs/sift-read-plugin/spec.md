## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Create `plugins/sift-read.lua` with hash-based caching, slice comparison, diff emission |

## ADDED Requirements

### Requirement: sift-read caches by hash

T2.1 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: First read caches hash
- **WHEN** T2.1 reads `"src/foo.rs"` for the first time
- **THEN** T2.1 SHALL compute sha256, store content via `sift.cache.store()`, cache the hash, and return the content.

#### Scenario: Repeat read returns unchanged
- **WHEN** T2.1 reads `"src/foo.rs"` again and the file hash matches the cached hash
- **THEN** T2.1 SHALL return `{ status = "unchanged", message = "[sift] src/foo.rs unchanged since last read" }`.

### Requirement: sift-read supports range reads

ALWAYS T2.1 SHALL accept optional offset and limit parameters.

#### Scenario: Range read with unchanged file
- **WHEN** T2.1 reads `"src/foo.rs"` with offset=5, limit=10 and the full file hash matches
- **THEN** T2.1 SHALL return `{ status = "unchanged", message = "[sift] src/foo.rs lines 5-14 unchanged" }`.

#### Scenario: Range read with changes outside range
- **WHEN** T2.1 reads `"src/foo.rs"` with offset=5, limit=10, the hash differs, but the slice 5-14 matches the cached content
- **THEN** T2.1 SHALL return `{ status = "unchanged", message = "[sift] src/foo.rs lines 5-14 unchanged; changes outside range" }`.

### Requirement: sift-read supports --fresh bypass

ALWAYS T2.1 SHALL accept `--fresh` as the first argument to bypass the cache and always return fresh content.

#### Scenario: Fresh read bypasses cache
- **WHEN** T2.1 reads `"src/foo.rs"` with `--fresh` flag and the file hash matches the cached hash
- **THEN** T2.1 SHALL return the file content, not "unchanged".

#### Scenario: Fresh read with range
- **WHEN** T2.1 reads `"src/foo.rs"` with `--fresh`, offset=5, limit=10
- **THEN** T2.1 SHALL return fresh content for lines 5-14, bypassing cache.

### Requirement: sift-read emits diffs

ALWAYS T2.1 SHALL compute a unified diff when the full file hash differs and scope is full.

#### Scenario: Diff emitted for changed file
- **WHEN** T2.1 reads `"src/foo.rs"` (full scope) and the hash differs
- **THEN** T2.1 SHALL return a unified diff between cached and current content, if the diff is significantly smaller than the full content.
