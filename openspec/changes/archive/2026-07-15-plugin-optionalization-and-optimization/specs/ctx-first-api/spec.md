# Ctx-First API — Consistent First Argument

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `ctx` as first argument to all sift.* functions in register_* methods |
| T1.2 | Update all built-in and optional plugins to pass ctx as first arg |
| T1.3 | Update all tests to use new signatures |

## ADDED Requirements

### Requirement: All sift.* functions accept ctx as first argument

ALWAYS T1.1 SHALL accept `ctx` (Lua table) as the first argument for every function in the `sift.*` namespace, including `sift.exec`, `sift.hash.*`, `sift.fs.*`, `sift.json.*`, `sift.toon.*`, `sift.jq.*`, `sift.env.*`, `sift.classify`, `sift.log`, `sift.log.nudge`, `sift.token_count`, `sift.store`.

#### Scenario: Ctx passed to hash function

- **WHEN** a plugin calls `sift.hash.sha256(ctx, "data")`
- **THEN** T1.1 SHALL accept the ctx parameter and ignore it (pure function, no side effects).

#### Scenario: Ctx passed to fs.read

- **WHEN** a plugin calls `sift.fs.read(ctx, "path", {offset=1, limit=10})`
- **THEN** T1.1 SHALL use ctx.cwd for relative path resolution if the path is not absolute.

### Requirement: All plugins updated

ALWAYS T1.2 SHALL update every .lua plugin file to pass ctx as the first argument to all sift.* calls.

#### Scenario: Updated plugin calls

- **WHEN** T7.2 is complete
- **THEN** every `sift.hash.sha256(data)` call SHALL be `sift.hash.sha256(ctx, data)`, and similarly for all other functions.
