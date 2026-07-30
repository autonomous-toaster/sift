# Tasks

## Phase 1: Nudge overhead fix

### 1.1 Fix nudge message length calculation in `json_shortest_impl()`
- Replace hardcoded `20` with actual nudge prefix length
- Use `"compressed output. raw: command cat ".len()` instead
- Location: `sift-core/src/lua/api_reg_io.rs`

## Phase 2: Veriplan plugin

### 2.1 Create `plugins/veriplan.lua`
- Match `veriplan` command pattern
- Only intercept `check` subcommand
- Ensure `--json` flag is present
- Use `sift.json.shortest()` with `{toon = true}`
- Return `streamed = true`
- Location: `plugins/veriplan.lua`

## Phase 3: Tests

### 3.1 Unit test for nudge overhead calculation
- Verify nudge message length calculation matches actual format string
- Location: `sift-core/src/lua/tests.rs`

### 3.2 Integration test for veriplan plugin
- Test `veriplan check` with and without `--json` flag
- Verify output is compressed (JSON or toon)
- Location: `sift/tests/cli.rs`
