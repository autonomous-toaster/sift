## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `sift.cache.store(hash, content)` — persist content by hash |
| T1.2 | Add `sift.cache.load(hash)` — load content by hash |

## ADDED Requirements

### Requirement: Store persists content by hash

T1.1 SHALL complete BEFORE T2.1 SHALL run.

#### Scenario: Content stored and retrievable
- **WHEN** T1.1 stores content `"hello"` with hash `"abc123"`
- **THEN** T1.1 SHALL write the content to `/tmp/sift/<session>/objects/sha256-abc123.txt`.

### Requirement: Load retrieves content by hash

T1.2 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: Content loaded by hash
- **WHEN** T1.2 loads hash `"abc123"` and the object exists
- **THEN** T1.2 SHALL return the stored content.

#### Scenario: Missing hash returns nil
- **WHEN** T1.2 loads hash `"nonexistent"`
- **THEN** T1.2 SHALL return nil.
