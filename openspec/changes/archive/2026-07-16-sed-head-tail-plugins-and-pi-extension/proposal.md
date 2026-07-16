## Why

The agent uses `sed -n`, `head`, and `tail` to read file ranges. These bypass sift's cache, wasting tokens on repeated reads. Plugins for these commands share the same range-aware cache as `sift-read` and `cat`. Additionally, a pi extension at `integrations/pi/sift.ts` replaces pi's built-in `bash` and `read` tools to route through sift, with `AI_SESSION` propagated on every call.

## What Changes

- **NEW**: `plugins/sed.lua` — intercepts `sed -n '<start>,<end>p' <path>` for range reads; passthrough for substitutions and other operations.
- **NEW**: `plugins/head.lua` — intercepts `head -n <count> <path>` for range reads.
- **NEW**: `plugins/tail.lua` — intercepts `tail -n <count> <path>` for range reads.
- **NEW**: `integrations/pi/sift.ts` — pi extension that overrides `read` and intercepts `bash` to route through sift, with `AI_SESSION` propagation and compaction cleanup.

## Capabilities

### New Capabilities
- `sed-plugin`: Range-aware `sed -n` interception with passthrough for non-range operations.
- `head-plugin`: Range-aware `head` interception.
- `tail-plugin`: Range-aware `tail` interception.
- `pi-extension`: Pi extension integrating sift as the bash/read backend.

## Impact

- **plugins/sed.lua**: New plugin.
- **plugins/head.lua**: New plugin.
- **plugins/tail.lua**: New plugin.
- **integrations/pi/sift.ts**: New pi extension.
