# Tasks

## Phase 1: Core Implementation

### 1.1 Add `split_first_command()` function
- Implement quote-aware scanning for `;`, `&&`, `||` outside quotes
- Return `(first_segment, rest)` tuple
- Reuse state machine pattern from `split_pipeline()`
- Location: `sift-core/src/lua/api.rs`

### 1.2 Modify `dispatch_full()` to use `split_first_command()`
- Before `shlex::split()`, check for metacharacters
- If found, split and use first segment for plugin matching
- Pass rest to `execute_passthrough()`
- Concatenate outputs from plugin and passthrough
- Location: `sift-core/src/lua/api.rs`

## Phase 2: Tests

### 2.1 Unit tests for `split_first_command()`
- Test `;` outside quotes → split correctly
- Test `&&` outside quotes → split correctly
- Test `||` outside quotes → split correctly
- Test metacharacters inside single quotes → no split
- Test metacharacters inside double quotes → no split
- Test escaped metacharacters → no split
- Test no metacharacters → returns (input, "")
- Location: `sift-core/src/lua/tests.rs`

### 2.2 Integration test for shell metacharacter handling
- Test `wc -c; echo $?` → `$?` expanded correctly
- Test `sed -n '1,5p' file; echo done` → sed plugin matches first segment
- Test `echo "hello; world"` → `;` inside quotes, no split
- Location: `sift/tests/cli.rs`
