## Why

sift-read cannot read PDFs or other binary document formats — `sift.fs.read()` returns binary garbage. AI agents frequently need to inspect PDFs (reports, papers, specs), and currently have no way to do so through sift. Adding document-to-text extraction via xberg closes this gap: sift-read transparently converts PDFs (and 97+ other formats) to clean Markdown, cached and diffed like any text file.

## What Changes

- **New `sift.ext` namespace** — extension hook for optional capabilities, detectable via nil check
- **New `sift.ext.mime` module** — always-available MIME type detection (magic bytes + extension), using `infer` and `mime_guess` crates
- **New `sift.ext.xberg` module** — document-to-text extraction via xberg crate, gated behind `xberg` feature flag
- **New `sift.ext.html` module** — HTML-to-Markdown conversion via `html-to-markdown-rs` crate, gated behind `html-md` feature flag
- **New `sift.ext.markdown` module** — Markdown compression via `mdmin` crate, gated behind `mdmin` feature flag
- **Modified `sift-read.lua`** — detect binary documents via MIME, route to xberg for extraction, fall back to raw read
- **Modified `curl.lua`** — detect document responses (PDF, HTML) via content-type, convert to Markdown via xberg/html-to-markdown, compress via mdmin
- **New Cargo features** — `xberg`, `xberg-office`, `xberg-ocr`, `html-md`, `mdmin` in sift-core
- **New Rust module** — `sift-core/src/lua/api_reg_ext.rs` for `sift.ext.*` API registration

## Capabilities

### New Capabilities
- `mime-detection`: MIME type detection from file paths and raw bytes, extension-to-MIME reverse lookup
- `xberg-extraction`: Document-to-text extraction for PDFs and 97+ formats, with configurable output format, page range, and OCR
- `html-conversion`: HTML-to-Markdown conversion with configurable options (heading style, link style, etc.)
- `markdown-compression`: Tree-sitter-based Markdown minification with 5 compression levels

### Modified Capabilities
- `file-reading`: sift-read SHALL detect binary document formats via MIME and transparently extract text via xberg when available

## Impact

- **sift-core**: New optional dependencies (xberg, html-to-markdown-rs, mdmin, infer, mime_guess). New `api_reg_ext.rs` module. New feature flags.
- **sift binary**: No change to binary size unless features are enabled. `xberg` feature adds ~5MB with PDF-only support.
- **plugins/sift-read.lua**: Modified to detect MIME and route to xberg. Backward compatible — falls through to raw read when xberg is unavailable or MIME is unsupported.
- **plugins/curl.lua**: Modified to detect HTML and document responses via content-type header, extract text via xberg or html-to-markdown, compress via mdmin.
- **No breaking changes**: All new APIs are additive. Existing behavior unchanged when features are disabled.
