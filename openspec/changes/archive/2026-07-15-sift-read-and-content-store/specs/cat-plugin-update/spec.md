## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Update `plugins/cat.lua` to use content store for cross-plugin cache sharing |

## MODIFIED Requirements

### Requirement: cat plugin shares cache with sift-read

T3.1 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: cat caches content for sift-read
- **WHEN** T3.1 reads `"src/foo.rs"` via `sift.exec()`
- **THEN** T3.1 SHALL store the content via `sift.cache.store()` and cache the hash, so that `sift-read` can detect it as unchanged.

#### Scenario: sift-read cache hit from cat
- **WHEN** T3.1 has cached `"src/foo.rs"` and T2.1 reads the same file
- **THEN** T2.1 SHALL detect the hash as cached and return "unchanged".
