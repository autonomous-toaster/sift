## Context

sift dispatches the last command in a pipeline to plugins via `try_pipeline()` in `api.rs`. For `curl ... | jq ...`, the curl command runs in bash and the jq plugin receives raw JSON as stdin. The `sift.jq.query()` API already exists using the in-process `jaq` library (`jaq-parse`, `jaq-syn`, `jaq-interpret` crates compiled into sift-core). No external `jaq` CLI is needed.

The curl plugin currently only checks `Content-Type` headers. GitHub serves `.md` and `.json` files as `text/plain`, so they bypass compression.

## Goals / Non-Goals

**Goals:**
- Intercept `jq` commands, apply filters via `sift.jq.query()` in-process
- Compress jq output with `sift.json.shortest()` (TOON, compact, etc.)
- Handle `-r` (raw output), `-c` (compact), `-n` (null input)
- Fall through to real `jq` for unsupported flags or file arguments
- Add URL extension fallback to curl plugin for `.md` → mdmin, `.json` → TOON
- Fix README `jaq` CLI reference

**Non-Goals:**
- `-s` (slurp) support in v1 — rare in agent sessions, can add later
- `--arg` / `--argjson` variable passing — complex, fall through to real `jq`
- SVG image stripping from HTML — separate concern

## Decisions

### 1. Plugin structure: single `jq.lua` with `sift.args.parse()`

Use `sift.args.parse()` for declarative argument parsing. Supported flags: `-r`, `-c`, `-n`, `-f`. Unknown flags → fall through to real `jq`.

### 2. Output compression: always `sift.json.shortest()`

After `sift.jq.query()` returns JSON, always pass through `sift.json.shortest(ctx, result, {toon = true})`. This tries TOON, compact JSON, and compacted JSON, picking the shortest. For `-r` output (raw strings), `shortest()` can't parse as JSON and returns unchanged — no special case needed.

### 3. `-r` handling: decode JSON array, extract values

`sift.jq.query()` always returns a JSON array. For `-r`, decode the array, extract each value as a string, join with `\n`. Then pass through `sift.json.shortest()` which returns the raw text unchanged.

### 4. `-n` handling: pass `"null"` as input

When `-n` is set, skip stdin and pass `"null"` as the JSON input to `sift.jq.query()`.

### 5. Extension fallback: after content-type, check URL

In `curl.lua`, after the content-type check fails to match JSON/HTML/document, extract the URL extension. If `.md` and `sift.ext.markdown` is available, apply mdmin. If `.json`, apply `sift.json.shortest()`.

### 6. No new crate dependencies

`jaq-parse`, `jaq-syn`, `jaq-interpret` are already in `sift-core/Cargo.toml`. The jq plugin uses existing `sift.jq.query()` and `sift.json.shortest()` APIs.

## Risks / Trade-offs

- **[Risk] `sift.jq.query()` may not support all jq filters** → Fall through to real `jq` on unsupported feature. The plugin knows its supported flags via `sift.args.parse()` with `allow_unknown = false` — unknown flags trigger passthrough before any filter execution.
- **[Risk] `-r` with objects produces multi-line JSON** → `sift.json.decode()` handles this correctly, extracting each value as a string. The raw text is not JSON, so `shortest()` returns it unchanged.
- **[Risk] Extension fallback may misidentify content** → Extension check is a fallback only when content-type is unrecognized. Content-type detection takes priority.

## Resolved Concerns

### Fallthrough conditions

The plugin falls through to real `jq` in exactly these cases:
- **Unknown flags**: Any flag not in the parser's spec (`-r`, `-c`, `-n`, `-f`) causes `sift.args.parse()` to return `nil, nil` → passthrough. This includes `--arg`, `--argjson`, `--slurp` (`-s`), `--from-file` (`-f`), and any other flag.
- **File arguments**: If stdin is nil and `-n` is not set, the plugin has no input → passthrough (real `jq` reads files).
- **Parse errors**: If `sift.args.parse()` returns `nil, err` (e.g., missing filter argument) → return error message.

### `-r` output format

`sift.jq.query()` always returns a JSON array, even for single-value filters. For `jq -r '.name'` with input `{"name":"John"}`, `sift.jq.query()` returns `["John"]`. The plugin decodes this array and extracts each value as a string. This is correct for all `-r` use cases — strings, numbers, objects, and arrays are all converted to their string representation and joined with newlines.

### Stdin buffering

The plugin reads all stdin into memory before processing. This is acceptable because:
- JSON piped to `jq` is typically small (API responses, file contents)
- The plugin falls through to real `jq` for large input (no explicit size limit — real `jq` handles streaming natively)
- Reading all stdin is required for `sift.jq.query()` which takes a single JSON value

### Output compression transparency

`sift.json.shortest()` stores the raw JSON and emits a nudge (`raw: 'command cat /tmp/sift/...'`), so users can always retrieve the original. Users who need standard JSON output can bypass sift entirely with `command jq ...`.

### Extension fallback specificity

The extension check only fires when content-type is unrecognized (not JSON, HTML, or document). This means:
- `Content-Type: application/json` + URL `.md` → JSON path (content-type wins)
- `Content-Type: text/plain` + URL `.json` → extension path (TOON)
- `Content-Type: text/plain` + URL `.md` → extension path (mdmin)
- `Content-Type: text/plain` + URL `.txt` → as-is (no extension match)

This is a low-risk heuristic that only activates when content-type gives no information.
