## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Add `sift.cache.set_path_hash(ctx, path, hash)` and `sift.cache.get_path_hash(ctx, path)` |

## ADDED Requirements

### Requirement: Path-to-hash mapping persists

T2.1 SHALL complete BEFORE T3.1 SHALL run.

#### Scenario: Set and get path hash
- **WHEN** T2.1 sets path hash for `"/tmp/test.txt"` to `"abc123"`
- **THEN** T2.1 SHALL return `"abc123"` when getting path hash for `"/tmp/test.txt"`.

#### Scenario: Unknown path returns nil
- **WHEN** T2.1 gets path hash for an unknown path
- **THEN** T2.1 SHALL return nil.
