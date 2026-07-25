## ADDED Requirements

### Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `strip_svg` option to `to_markdown()` Lua function |
| T1.2 | Enable `strip_svg` in `curl.lua` for HTML responses |

### Requirement: strip_svg option on to_markdown()

`sift.ext.html.to_markdown()` MUST accept a `strip_svg` boolean option. When `true`, the converter MUST exclude `<svg>` and `<math>` elements (and all their descendants) from the output.

T1.1 SHALL complete BEFORE T1.2 SHALL run.

#### Scenario: strip_svg=true excludes SVG elements
- **WHEN** `to_markdown(html, {strip_svg=true})` is called with HTML containing `<svg>...</svg>`
- **THEN** the output MUST NOT contain any base64 data URI from the SVG element

#### Scenario: strip_svg=false preserves SVG (backward compat)
- **WHEN** `to_markdown(html)` or `to_markdown(html, {})` is called with HTML containing `<svg>...</svg>`
- **THEN** the output MUST contain the SVG as a base64 data URI (existing behavior)

#### Scenario: strip_svg=true excludes MathML elements
- **WHEN** `to_markdown(html, {strip_svg=true})` is called with HTML containing `<math>...</math>`
- **THEN** the output MUST NOT contain any base64 data URI from the MathML element

### Requirement: curl plugin enables strip_svg

`plugins/curl.lua` MUST pass `{strip_svg=true}` to `to_markdown()` when processing HTML responses.

T1.2 SHALL complete AFTER T1.1 SHALL complete.

#### Scenario: curl HTML response strips SVG
- **WHEN** `curl` fetches an HTML page with inlined `<svg>` elements
- **THEN** the plugin output MUST NOT contain base64-encoded SVG data URIs
