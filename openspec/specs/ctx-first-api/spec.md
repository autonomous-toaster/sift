# Ctx-First API

## Purpose

Ensure all `sift.*` functions accept `ctx` as the first argument for API consistency and future-proofing.

## Requirements

### Requirement: All sift.* functions accept ctx as first argument

The system SHALL accept `ctx` (Lua table) as the first argument for every function in the `sift.*` namespace, including `sift.exec`, `sift.hash.*`, `sift.fs.*`, `sift.json.*`, `sift.toon.*`, `sift.jq.*`, `sift.env.*`, `sift.classify`, `sift.log`, `sift.log.nudge`, `sift.token_count`, `sift.store`.

#### Scenario: Ctx passed to hash function

- **WHEN** a plugin calls `sift.hash.sha256(ctx, "data")`
- **THEN** the system SHALL accept the ctx parameter and ignore it (pure function, no side effects).

#### Scenario: Ctx passed to fs.read

- **WHEN** a plugin calls `sift.fs.read(ctx, "path", {offset=1, limit=10})`
- **THEN** the system SHALL use ctx.cwd for relative path resolution if the path is not absolute.

### Requirement: All plugins updated

The system SHALL update every .lua plugin file to pass ctx as the first argument to all sift.* calls.

#### Scenario: Updated plugin calls

- **WHEN** implementation is complete
- **THEN** every `sift.hash.sha256(data)` call SHALL be `sift.hash.sha256(ctx, data)`, and similarly for all other functions.
