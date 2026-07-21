## Context

sift is a Rust shell proxy with an embedded Lua plugin runtime (mlua). Plugins intercept commands and transform output. sift-read.lua reads files, caches by SHA256 hash, and returns content or unified diffs. Currently it reads all files as text strings — binary files (PDFs, Office docs) produce garbage.

The `sift.*` Lua API is registered in `sift-core/src/lua/api.rs` via `register_sift_table()`. Each sub-module (sift.fs, sift.cache, sift.json, etc.) is registered in its own file under `sift-core/src/lua/`. New `sift.ext.*` modules will follow the same pattern in a new `api_reg_ext.rs`.

## Goals / Non-Goals

**Goals:**
- Add `sift.ext` namespace as an extension hook for optional capabilities
- Add `sift.ext.mime` for MIME type detection (always available)
- Add `sift.ext.xberg` for document-to-text extraction (gated behind `xberg` feature)
- Add `sift.ext.html` for HTML-to-Markdown conversion (gated behind `html-md` feature)
- Add `sift.ext.markdown` for Markdown compression (gated behind `mdmin` feature)
- Modify sift-read.lua to detect binary documents and route to xberg
- All new APIs detectable via nil check (`if sift.ext.xberg ~= nil then`)

**Non-Goals:**
- Full async plugin dispatch (deferred to future change)
- Streaming extraction for large PDFs (deferred to v2)
- OCR or ML-based features (require xberg-ocr feature, not in v1)
- MCP server integration (deferred)

## Decisions

### D1: sift.ext namespace over flat sift.*
`sift.ext` signals "optional extension" vs core API. Future extensions (ocr, audio, code) live here. Lua detects availability via nil check: `if sift.ext.xberg ~= nil then`.

### D2: MIME detection via infer + mime_guess (always available)
`infer` (0.22) for magic byte detection, `mime_guess` (2.0) for extension-to-MIME. Both are lightweight (~50KB combined). Independent of xberg — curl plugin and other consumers can use MIME detection without pulling in document extraction deps.

### D3: xberg crate with pdf feature only
xberg's `pdf` feature pulls in pdf_oxide, lopdf, image (png), flate2, html-to-markdown-rs. No OCR, no ML, no transcription. Binary size increase ~5MB. Sub-features (`xberg-office`, `xberg-ocr`) available for users who need more formats.

### D4: Sync wrapper for async xberg API
xberg's `extract()` is async. sift's Lua dispatch is sync. Use `tokio::runtime::Handle::current().block_on()` to bridge — sift already has a tokio runtime for cache operations. This is acceptable because PDF extraction is I/O-bound (1-5s), and the blocking happens inside the plugin, not in the dispatch path.

### D5: Cache keyed by file hash, not xberg version
Same approach as existing sift-read: hash the raw file bytes, cache extracted text by that hash. xberg version changes don't invalidate cache — user runs `sift-read --fresh` to force re-extraction. Consistent with existing behavior.

### D6: Error fallback to raw binary read
If xberg fails (corrupt PDF, password, timeout), sift-read falls back to `sift.fs.read(path)` — returns raw binary. Agent gets garbage but doesn't crash. User can investigate with `--fresh` or `command cat`.

### D7: Single generic extract() over format-specific functions
Council voted for Option A: one `sift.ext.xberg.extract(path, opts)` handles all 97 formats. Format-specific shortcuts (e.g., `sift.ext.pdf.to_markdown`) can be added later if demand arises.

### D8: URL-derived slug for stored curl responses
`sift.store()` slugs for curl responses SHALL be derived from the URL (last path segment) with content-type-based fallback. The URL is extracted as the last non-flag argument from curl args. When the URL-derived slug has no file extension, the extension from the response content-type SHALL be appended. When no filename is present in the URL, the slug SHALL be `response_<mime>` with `/` replaced by `_`.

### D9: Unified nudge format for raw storage
All three raw-storage nudge patterns SHALL use the same format: `raw: 'command cat <path>'`. This applies to `sift.json.shortest()`, `sift.exec()` error path, and `sift.store()`. The format is the most concise and already established by `json.shortest()`.

## Risks / Trade-offs

- **[Risk] xberg dependency weight** → Mitigation: optional feature flag, pdf-only feature, no impact on default builds
- **[Risk] xberg API instability** → Mitigation: pin xberg version in Cargo.toml, upgrade deliberately
- **[Risk] Blocking on async in sync dispatch** → Mitigation: acceptable for I/O-bound extraction (1-5s). Full async dispatch deferred to future change.
- **[Risk] Cache staleness on xberg upgrade** → Mitigation: `--fresh` flag exists. No automatic invalidation — consistent with existing behavior.
- **[Trade-off] mdmin uses tree-sitter** → Heavy dep (~10MB) for Markdown parsing. Gated behind `mdmin` feature. Only users who want Markdown compression pay for it.
