## Why

Agents frequently pipe JSON output through `jq` to extract specific fields (e.g., `curl ... | jq '.results[] | {title, url}'`). Currently this spawns a subprocess and the output is returned as plain text — missing the opportunity to compress with TOON. A jq plugin that uses the in-process `jaq` library and compresses output with `sift.json.shortest()` would save tokens and avoid subprocess overhead.

Additionally, the curl plugin misses `.md` and `.json` files served as `text/plain` (e.g., GitHub raw content), and the README incorrectly references a `jaq` CLI dependency.

## What Changes

- **New**: `jq.lua` plugin — intercepts `jq` commands, uses `sift.jq.query()` in-process, compresses output with `sift.json.shortest()`
- **Modified**: `curl.lua` — add URL extension fallback for `.md` (mdmin) and `.json` (TOON) when content-type is unrecognized
- **Fixed**: `README.md` — remove incorrect `jaq` CLI dependency reference

## Capabilities

### New Capabilities
- `jq-plugin`: Intercept `jq` commands, apply filters via in-process `jaq` library, and compress output with TOON or shortest JSON format

### Modified Capabilities
*(none — no existing spec-level behavior changes)*

## Impact

- **New file**: `plugins/jq.lua`
- **Modified file**: `plugins/curl.lua` (extension fallback)
- **Modified file**: `README.md` (remove jaq CLI reference)
- **Dependencies**: `jaq-parse`, `jaq-syn`, `jaq-interpret` already in `sift-core/Cargo.toml` — no new crate deps
