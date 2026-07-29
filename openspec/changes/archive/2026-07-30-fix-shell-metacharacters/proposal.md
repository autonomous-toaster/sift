## Why

`shlex::split()` is a word splitter, not a shell parser. It doesn't handle shell metacharacters like `;`, `&&`, `||` — it treats them as literal characters. So `wc -c; echo $?` is split as `["wc", "-c;", "echo", "$?"]` instead of recognizing `;` as a command separator. This causes plugins that reconstruct commands from args (rtk, jq, etc.) to produce incorrect commands with metacharacters inside single quotes, breaking shell variable expansion and command sequencing.

## What Changes

In `dispatch_full()`, before using `shlex::split()`, scan for shell metacharacters (`;`, `&&`, `||`) outside of quotes. If found, split the command on the metacharacter, use only the first segment for plugin matching, and pass remaining segments directly to bash.

## Capabilities

### New Capabilities
- `shell-metacharacter-handling`: Quote-aware detection and splitting of shell metacharacters in `dispatch_full()`

### Modified Capabilities
- (none)

## Impact

**sift-core/src/lua/api.rs:**
- New function `split_first_command()` — splits on `;`, `&&`, `||` outside quotes, returns (first_segment, rest)
- Modified `dispatch_full()` — uses `split_first_command()` before `shlex::split()`
- Reuses quote-tracking logic from existing `split_pipeline()`

**Tests:**
- Unit tests for `split_first_command()` with various metacharacter combinations
- Integration test for `wc -c; echo $?` verifying `$?` expansion
- Integration test for `sed -n '1,5p' file; echo done` verifying plugin matching on first segment
