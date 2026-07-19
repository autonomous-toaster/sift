# git-commit-hook-and-curl-optimizer

## Why

Two behavioral plugins that optimize common agent workflows: (1) prevent accidental hook bypass in `git commit`, and (2) automatically compress JSON responses from `curl` to reduce token waste while preserving access to raw data.

## What Changes

- **New plugin `git-commit.lua`**: Intercepts `git commit` commands, detects `-n`/`--no-verify` flags, returns non-zero exit code with nudge explaining why hooks must run. All other git commands passthrough to rtk.
- **New plugin `curl.lua`**: Intercepts `curl` commands. If `-v`/`--verbose` not requested, adds `-w "\n%{content_type}"` to detect JSON responses. JSON bodies are compressed via `sift.json.shortest()` with raw data stored and nudge emitted. Non-JSON responses returned as-is. If `-v` explicitly requested, runs as-is with full output. Curl exit code always propagated.

## Capabilities

### New Capabilities

- `git-commit-hook`: Behavioral plugin forbidding `-n`/`--no-verify` on `git commit`
- `curl-json-optimizer`: Plugin that auto-detects and compresses JSON curl responses

### Modified Capabilities

None.

## Impact

- Two new Lua plugin files in `plugins/` directory
- No core code changes needed — pure plugin implementations
- No new dependencies
