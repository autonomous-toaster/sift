## 1. Add `strip_svg` option to `to_markdown()` Lua function

- [x] 1.1 Add `strip_svg` boolean option parsing in the `to_markdown` closure in `api_reg_ext.rs` — when `true`, set `options.exclude_selectors = vec!["svg".into(), "math".into()]`
- [x] 1.2 Add unit test for `strip_svg=true` excluding SVG elements, and `strip_svg=false` preserving existing behavior

## 2. Enable `strip_svg` in curl plugin

- [x] 2.1 Update `plugins/curl.lua` to pass `{strip_svg=true}` to `sift.ext.html.to_markdown()` in the HTML content-type branch
