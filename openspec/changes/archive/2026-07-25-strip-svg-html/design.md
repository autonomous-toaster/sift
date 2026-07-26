## Context

The `sift.ext.html.to_markdown()` Lua function wraps `html-to-markdown-rs::convert()` with a thin options layer. Currently it exposes `heading_style` and `link_style` options. The underlying crate supports `exclude_selectors: Vec<String>` in `ConversionOptions` — CSS selectors for elements to drop entirely (element + all descendants) from the output.

When `curl` fetches an HTML page, it calls `to_markdown(body)` with default options. Inlined `<svg>` and `<math>` elements are serialized as base64 data URIs (`![SVG Image](data:image/svg+xml;base64,...)`), which can be tens of KB each and pollute the context window.

## Goals / Non-Goals

**Goals:**
- Add a `strip_svg` boolean option to `sift.ext.html.to_markdown()` that strips `<svg>` and `<math>` elements
- Enable it by default in `plugins/curl.lua` for HTML responses
- Zero new dependencies — use the crate's existing `exclude_selectors` mechanism

**Non-Goals:**
- Not stripping `<img src="data:...">` (data URI images are rare in practice; can be added later if needed)
- Not adding a general-purpose `exclude_selectors` Lua API (callers that need custom selectors can use a future enhancement)
- Not modifying the `html-to-markdown-rs` crate itself

## Decisions

1. **Use `exclude_selectors` over custom state machine** — The crate already supports CSS selectors for element exclusion. This is more robust than a hand-rolled byte scanner and handles all edge cases (nesting, case sensitivity, CDATA, comments, self-closing tags, quoted attributes) for free.

2. **Option name `strip_svg`** — Concise, self-explanatory, and leaves room for future `strip_*` options. Defaults to `false` for backward compatibility.

3. **Strip both `<svg>` and `<math>`** — MathML has the same problem (serialized as base64 data URIs). Stripping both under one flag avoids proliferation of per-element toggles.

4. **Enable in `curl.lua` only** — The curl plugin is the primary consumer for web search. Other callers of `to_markdown()` can opt in explicitly.

## Risks / Trade-offs

- **False positives on `<svg>` used as inline icons in documentation** — Some pages use SVG for simple icons (e.g., GitHub's octicons). These are replaced with nothing, losing the visual cue. Acceptable for web search where the text content is what matters.
- **`exclude_selectors` is a crate feature, not a Lua API** — If a caller needs custom selectors, they'd need a Rust change. Acceptable for now; can be exposed later.
- **`strip_svg` defaults to `false`** — Existing callers are unaffected, but the curl plugin must explicitly opt in. This is intentional — the option is opt-in by design.
