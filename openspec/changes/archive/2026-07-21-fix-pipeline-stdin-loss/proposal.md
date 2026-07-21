## Why

Sift's pipeline optimization silently drops stdin when the last segment's plugin returns `passthrough`, and incorrectly triggers on wildcard plugins. This causes commands like `find ... | sort` to produce empty output — the preceding segments run in bash, but the final command receives no input. The agent gets no output, no error, and no indication that data was lost.

## What Changes

- **Pipeline optimization skips wildcard plugins**: `try_pipeline()` will not trigger when the matched plugin has `pattern = "*"`. The full pipeline runs in bash instead.
- **`execute_passthrough` accepts stdin**: When a specific plugin returns `passthrough` in pipeline mode, the accumulated stdin from preceding segments is forwarded to the passthrough command.
- **`dispatch()` passes stdin to `execute_passthrough`**: The stdin parameter received by `dispatch()` is forwarded when falling through to passthrough execution.

## Capabilities

### New Capabilities

- `pipeline-stdin-preservation`: Ensures stdin from preceding pipeline segments is preserved when the last segment's plugin returns `passthrough`, and prevents wildcard plugins from triggering pipeline optimization.

### Modified Capabilities

*(None — no existing spec-level requirements are changing.)*

## Impact

- `sift-core/src/lua/api.rs`: `try_pipeline()`, `execute_passthrough()`, and `dispatch()` — the pipeline optimization logic and passthrough execution path.
- No API changes to plugins. No changes to the Lua plugin interface.
- No new dependencies.
