## 1. Foundation — Dependencies and sift.ext namespace

- [x] 1.1 Add `infer` and `mime_guess` dependencies to sift-core/Cargo.toml. Create `sift-core/src/lua/api_reg_ext.rs` with `register_sift_ext()` function. Implement `sift.ext.mime` module with `detect()`, `detect_bytes()`, and `extension()` functions. Add unit tests for MIME detection from paths and bytes.
- [x] 1.2 Add `xberg` dependency to sift-core/Cargo.toml with `pdf` and `tokio-runtime` features behind `xberg` feature flag. Implement `sift.ext.xberg` module with `extract()`, `extract_bytes()`, and `is_supported()` functions. Use `tokio::runtime::Handle::current().block_on()` to bridge async xberg API. Add unit tests for PDF extraction and format detection.
- [x] 1.3 Add `html-to-markdown-rs` dependency to sift-core/Cargo.toml behind `html-md` feature flag. Implement `sift.ext.html` module with `to_markdown()` function. Add unit tests for HTML conversion with various options.
- [x] 1.4 Add `mdmin` dependency to sift-core/Cargo.toml behind `mdmin` feature flag. Implement `sift.ext.markdown` module with `compress()` function. Add unit tests for compression at different levels.
- [x] 1.5 Wire `register_sift_ext()` into `register_sift_table()` in `sift-core/src/lua/api.rs`. Each sub-module is registered only when its feature flag is enabled. Add integration test verifying nil check: `sift.ext.xberg` is nil when feature disabled, non-nil when enabled.

## 2. Integration — Plugin modifications

- [x] 2.1 Modify `plugins/sift-read.lua` to detect binary documents: call `sift.ext.mime.detect(path)` before reading, check `sift.ext.xberg.is_supported(mime)` when xberg is available, route to `sift.ext.xberg.extract()` for supported formats, cache extracted text by file hash, fall back to `sift.fs.read()` otherwise. Add unit tests for PDF routing, caching, and fallback behavior.
- [x] 2.2 Modify `plugins/curl.lua` to detect HTML and document responses via content-type header: convert HTML to Markdown via `sift.ext.html.to_markdown()`, optionally compress via `sift.ext.markdown.compress()`, extract PDF/documents via `sift.ext.xberg.extract()` with temp file, store raw response for re-read.
- [x] 2.3 Fix curl plugin slug: extract URL from args (last non-flag argument), derive slug from URL path (last segment), fall back to `response_<mime>` when no filename, append extension from content-type when slug has none.
- [x] 2.4 Unify nudge format for raw storage: change `sift.exec()` error nudge from `"use 'command cat {path}' for raw output"` to `"raw: 'command cat {path}'"` in `api_reg_cache.rs`. Change `sift.store()` nudge from `"stored: 'command cat {path_str}'"` to `"raw: 'command cat {path_str}'"` in `api_reg_io.rs`.

## 3. Verification — End-to-end validation

- [x] 3.1 Verify MIME detection works for PDF, PNG, JPEG, HTML, and plain text files via both extension and magic bytes.
- [x] 3.2 Verify xberg extraction produces correct Markdown output from a test PDF, with configurable format and page range.
- [x] 3.3 Verify sift-read routes PDFs to xberg, caches extracted text, returns "unchanged" on repeat read, and falls back to raw read when xberg is unavailable.
- [x] 3.4 Verify feature flag gating: `sift.ext.xberg`, `sift.ext.html`, `sift.ext.markdown` are nil when their respective features are disabled.
