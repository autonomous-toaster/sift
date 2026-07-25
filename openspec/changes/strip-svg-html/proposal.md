## Why

When `curl` fetches an HTML page that contains inlined `<svg>` or `<math>` elements, the `html-to-markdown-rs` converter serializes them as base64-encoded data URIs in the markdown output:

```
![SVG Image](data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI...
```

These base64 blobs can be tens of kilobytes each, and a single web page may contain dozens of SVG icons, charts, or diagrams. This inflates token consumption, pollutes the context window with non-semantic data, and degrades the quality of web search results for AI agents.

The `html-to-markdown-rs` crate already provides `exclude_selectors` in `ConversionOptions` — a built-in mechanism to drop entire elements and their descendants via CSS selectors. This change wires that option through to the Lua `sift.ext.html.to_markdown()` API so callers can strip SVG/MathML without any custom parsing code.

## What Changes

- Add a `strip_svg` boolean option to `sift.ext.html.to_markdown(html, opts?)`
- When `true`, set `options.exclude_selectors = ["svg", "math"]` before conversion
- Update `plugins/curl.lua` to pass `{strip_svg=true}` when converting HTML responses
- No new dependencies — the `exclude_selectors` feature is already in `html-to-markdown-rs` v3

## Capabilities

### New Capabilities
- `strip-svg-html`: Strip inlined `<svg>` and `<math>` elements from HTML before markdown conversion, replacing them with nothing (elements and all descendants are dropped). Controlled via a `strip_svg` option on `sift.ext.html.to_markdown()`.

### Modified Capabilities
- (none)

## Impact

- **`sift-core/src/lua/api_reg_ext.rs`**: ~5 lines added to the `to_markdown` closure to read the `strip_svg` option and set `exclude_selectors`
- **`plugins/curl.lua`**: ~1 line change to pass `{strip_svg=true}` to `to_markdown()`
- **Dependencies**: None added
- **Backward compatibility**: `strip_svg` defaults to `false` — existing callers unaffected
