## ADDED Requirements

### Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Update cat.lua nudge messages (3 messages) |
| T1.2 | Update head.lua nudge messages (2 messages) |
| T1.3 | Update tail.lua nudge messages (2 messages) |
| T1.4 | Update sed.lua nudge messages (2 messages) |
| T1.5 | Update sift-read.lua nudge messages (5 messages) |
| T1.6 | Update rtk.lua and jq.lua nudge messages (3 messages) |
| T2.1 | Update Rust nudge messages in api.rs, api_reg_io.rs, api_reg_cache.rs |
| T3.1 | Update test assertions for new nudge format |

---

### Requirement: plugin nudge messages use new format

All plugin nudge messages SHALL use the new format: unchanged messages use "already in your context", compressed messages use "compressed output. raw:", binary messages drop feature instructions.

#### SHALL: cat.lua nudges updated

T1.1 SHALL update cat.lua's 3 nudge messages. "unchanged (cached)" SHALL become "unchanged — already in your context". "raw: command cat" SHALL become "compressed output. raw: command cat". "bypass if stale" SHALL be removed.

#### SHALL: head.lua nudges updated

T1.2 SHALL update head.lua's 2 nudge messages. "unchanged (cached)" SHALL become "unchanged — already in your context". "bypass if stale" SHALL be removed.

#### SHALL: tail.lua nudges updated

T1.3 SHALL update tail.lua's 2 nudge messages. Same format changes as head.lua.

#### SHALL: sed.lua nudges updated

T1.4 SHALL update sed.lua's 2 nudge messages. Same format changes as head.lua.

#### SHALL: sift-read.lua nudges updated

T1.5 SHALL update sift-read.lua's 5 nudge messages. "unchanged (cached)" SHALL become "unchanged — already in your context". "raw: sift-read --raw" SHALL become "compressed output. raw: sift-read --raw". Binary document nudge SHALL drop feature installation instructions.

#### SHALL: rtk.lua and jq.lua nudges updated

T1.6 SHALL update rtk.lua and jq.lua. "raw: command" SHALL become "compressed output. raw: command".

---

### Requirement: Rust nudge messages use new format

Rust-side nudge messages in api.rs, api_reg_io.rs, and api_reg_cache.rs SHALL use the new format.

#### SHALL: burst warning updated

T2.1 SHALL update the burst warning in api.rs from parenthetical format to "Result is stable — file hasn't changed on disk. Same output until it does."

#### SHALL: IO nudges updated

T2.1 SHALL update api_reg_io.rs nudges from "raw: command cat" to "output saved. raw: command cat" (store) and "compressed output. raw: command cat" (JSON shortest).

#### SHALL: error save nudge updated

T2.1 SHALL update api_reg_cache.rs nudge from "raw: command cat" to "error output saved. raw: command cat".

---

### Requirement: test assertions match new format

Test assertions that check nudge message content SHALL be updated to match the new format.

#### SHALL: unit test assertions updated

T3.1 SHALL update test assertions in tests.rs and tests_plugins.rs that reference old nudge strings.

#### SHALL: integration test assertions updated

T3.1 SHALL update integration test assertions in cli.rs that reference old nudge strings.
