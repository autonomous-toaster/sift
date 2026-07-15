## Task Reference

| Task ID | Description |
|---------|-------------|
| T4.1 | Add automatic pruning of objects older than max age |
| T4.2 | Update `plugins/reset.lua` to clear the content store |

## ADDED Requirements

### Requirement: Old objects are pruned automatically

ALWAYS T4.1 SHALL delete objects in `/tmp/sift/<session>/objects/` older than a configurable max age (default 24h).

#### Scenario: Old object pruned
- **WHEN** T4.1 runs and an object's mtime exceeds the max age
- **THEN** T4.1 SHALL delete that object file.

### Requirement: Reset clears content store

T4.2 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: Reset removes all objects
- **WHEN** T4.2 runs the reset plugin
- **THEN** T4.2 SHALL delete all objects in `/tmp/sift/<session>/objects/` and clear all cached hashes for the session.
