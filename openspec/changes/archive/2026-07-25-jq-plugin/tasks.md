## 1. jq Plugin

- [x] 1.1 Create `plugins/jq.lua` with argument parsing via `sift.args.parse()`, supporting `-r`, `-c`, `-n`, `-f` flags and falling through on unknown flags
- [x] 1.2 Implement piped stdin handling: read all stdin, pass to `sift.jq.query()` with the parsed filter, handle `-n` (null input) and no-stdin (passthrough) cases
- [x] 1.3 Implement output compression: pass `sift.jq.query()` result through `sift.json.shortest(ctx, result, {toon = true})`, handle `-r` by decoding JSON array to raw strings before shortest()
- [x] 1.4 Add plugin test cases: basic filter, `-r` raw output, `-n` null input, unknown flag fallthrough, unsupported feature fallthrough

## 2. Curl Extension Fallback

- [x] 2.1 Add URL extension extraction to `plugins/curl.lua` after content-type check fails: check for `.md` → mdmin, `.json` → TOON
- [x] 2.2 Add test cases: `.md` file served as text/plain, `.json` file served as text/plain, content-type takes priority over extension

## 3. README Fix

- [x] 3.1 Remove line 530 in `README.md` referencing optional `jaq` CLI for `sift.jq.query()`
