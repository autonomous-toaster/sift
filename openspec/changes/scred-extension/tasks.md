## 1. Dependency and feature gate

- [x] 1.1 Add `scred-redactor` git dependency and `scred` feature to `sift-core/Cargo.toml`

## 2. Modify RedactionStream in scred source

- [x] 2.1 Add `PatternSelector` field to `RedactionStream` in `scred-redactor/src/streaming/mod.rs` — add constructor parameter defaulting to `All`, filter matches in `feed()` before `redact_in_place()`

## 3. Register sift.ext.scred extension

- [x] 3.1 Add `register_ext_scred()` in `sift-core/src/lua/api_reg_ext.rs` — register `sift.ext.scred` module gated behind `#[cfg(feature = "scred")]`
- [x] 3.2 Implement `create_transform()` — create `Arc<Mutex<RedactionStream>>`, return two Lua functions (feed + finalize) sharing it via `Arc`
- [x] 3.3 Implement `redact()` one-shot convenience — create stream, feed entire text, finalize, return result
- [x] 3.4 Implement opts parsing for pattern selection — parse `redact` field from opts table into `PatternSelector`

## 4. Create user bash plugin

- [x] 4.1 Create `shell.lua` with `pattern = "__default__"`, `priority = -500` — use `sift.ext.scred.create_transform()` for streaming redaction, fall through when scred unavailable

## 5. Remove old scred plugin

- [x] 5.1 Delete `plugins/scred.lua`

## 6. Tests

- [x] 6.1 Add unit tests for `sift.ext.scred` transform and one-shot APIs in `sift-core/src/lua/tests_ext.rs`
- [x] 6.2 Add integration test for bash plugin with scred in `sift/tests/cli.rs`
