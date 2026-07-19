## 1. Rewrite sift.toon.encode and decode

- [x] 1.1 Rewrite `sift.toon.encode` — accept `(data, options?)`, map `delimiter`/`indent` to `EncodeOptions`, fall back to `encode_default` when no options
- [x] 1.2 Rewrite `sift.toon.decode` — accept `(str, options?)`, map `strict`/`no_coerce` to decode variant, error if both set, fall back to `decode_default`
- [x] 1.3 Drop `ctx` parameter from both functions — pure functions like `sift.str.*`

## 2. Tests

- [x] 2.1 Update smoke test in `tests_plugins.rs` — verify new signatures (no ctx, options table)
- [x] 2.2 Verify no shipped plugins break — check `cat.lua`, `sift-read.lua` for `sift.toon.*` calls

## 3. Fix openspec double output

- [x] 3.1 Add `silent` option to `sift.exec` — suppress stdout/stderr printing when `silent = true`
- [x] 3.2 Update `openspec.lua` to use `sift.exec(ctx, cmd, {silent = true})` — prevents raw JSON from being printed alongside TOON output