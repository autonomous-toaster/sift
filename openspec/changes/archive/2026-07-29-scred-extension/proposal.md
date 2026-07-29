## Why

The current scred plugin (`plugins/scred.lua`) only intercepts `echo`, `env`, and `printenv` commands. Secrets can leak through any command — `git log`, `curl`, `cat ~/.aws/credentials`, or any output that happens to contain environment variable values. A per-command plugin approach doesn't scale.

sift already has a streaming transform mechanism in `sift.exec()` (the `transform` option processes stdout chunks in real-time). By embedding scred's `RedactionStream` as a `sift.ext.scred` Rust extension, any plugin can transparently redact secrets from command output. A user bash plugin overriding `__default__` would apply redaction to ALL commands by default.

## What Changes

- Add `scred-redactor` as an optional dependency of `sift-core` (gated behind `scred` feature)
- Register `sift.ext.scred` Lua extension with streaming transform API
- Modify scred's `RedactionStream` to support `PatternSelector` for configurable redaction
- Create user bash plugin (`shell.lua`) that wraps all command output through scred
- Remove the old `plugins/scred.lua` (replaced by the bash plugin + extension)

## Capabilities

### New Capabilities
- `scred-extension`: Embed scred's secret redaction engine as `sift.ext.scred` with streaming transform API, configurable pattern selection, and a user bash plugin that redacts all command output

### Modified Capabilities
- (none)

## Impact

**sift-core:**
- New optional dependency: `scred-redactor` (pulls in `scred-detector` with pure Rust SIMD patterns)
- New feature flag: `scred`
- New file: `sift-core/src/lua/api_reg_ext.rs` additions for `register_ext_scred()`

**scred (external):**
- Small change to `RedactionStream` in `scred-redactor` to accept `PatternSelector`

**Plugins:**
- Remove `plugins/scred.lua`
- Create user bash plugin at `~/.config/sift/plugins/shell.lua` (documented, not shipped in repo)

**Performance:**
- Streaming redaction adds per-chunk overhead (SIMD pattern matching on each chunk)
- Lookahead window (512B) adds latency equal to 512B of output
- Acceptable for interactive use; measurable but small for typical command output
