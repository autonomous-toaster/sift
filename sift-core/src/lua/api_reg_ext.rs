//! Extension API registration for `sift.ext.*` modules.
//!
//! Each sub-module is registered only when its Cargo feature flag is enabled.
//! Lua detects availability via nil check: `if sift.ext.xberg ~= nil then`.

use super::SiftLua;
use anyhow::Result;
use mlua::Table;

impl SiftLua {
    /// Register all `sift.ext.*` extension modules.
    pub(super) fn register_sift_ext(&self, sift: &Table) -> Result<()> {
        let ext = self.lua.create_table()?;
        self.register_ext_mime(&ext)?;
        #[cfg(feature = "xberg")]
        self.register_ext_xberg(&ext)?;
        #[cfg(feature = "html-md")]
        self.register_ext_html(&ext)?;
        #[cfg(feature = "mdmin")]
        self.register_ext_markdown(&ext)?;
        sift.set("ext", ext)?;
        Ok(())
    }

    /// Register `sift.ext.mime` — MIME type detection (always available).
    #[allow(clippy::too_many_lines)]
    fn register_ext_mime(&self, ext: &Table) -> Result<()> {
        let mime = self.lua.create_table()?;

        // sift.ext.mime.detect(path) -> string
        let detect = self
            .lua
            .create_function(|_, (_ctx, path): (Table, String)| {
                // Try extension first
                let ext = std::path::Path::new(&path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_lowercase);
                if let Some(ext_str) = ext {
                    if let Some(mime_type) = mime_guess::from_ext(&ext_str).first_raw() {
                        return Ok(mime_type.to_string());
                    }
                }
                // Fall back to magic byte detection
                let Ok(bytes) = std::fs::read(&path) else {
                    return Ok("application/octet-stream".to_string());
                };
                Ok(infer::get(&bytes)
                    .map_or("application/octet-stream", |k| k.mime_type())
                    .to_string())
            })?;
        mime.set("detect", detect)?;

        // sift.ext.mime.detect_bytes(bytes) -> string
        let detect_bytes =
            self.lua
                .create_function(|_, (_ctx, bytes): (Table, mlua::String)| {
                    let raw: &[u8] = &bytes.as_bytes();
                    Ok(infer::get(raw)
                        .map_or("application/octet-stream", |k| k.mime_type())
                        .to_string())
                })?;
        mime.set("detect_bytes", detect_bytes)?;

        // sift.ext.mime.extension(mime) -> string
        let extension = self
            .lua
            .create_function(|_, (_ctx, mime_str): (Table, String)| {
                let ext = mime_guess::get_mime_extensions_str(&mime_str)
                    .and_then(|exts| exts.first().copied())
                    .unwrap_or("");
                Ok(ext.to_string())
            })?;
        mime.set("extension", extension)?;

        ext.set("mime", mime)?;
        Ok(())
    }

    /// Register `sift.ext.xberg` — document-to-text extraction (gated behind `xberg` feature).
    #[cfg(feature = "xberg")]
    fn register_ext_xberg(&self, ext: &Table) -> Result<()> {
        let xberg_tbl = self.lua.create_table()?;

        // sift.ext.xberg.extract(path, opts?) -> string
        let extract =
            self.lua
                .create_function(|_, (_ctx, path, opts): (Table, String, Option<Table>)| {
                    let mut config = xberg::ExtractionConfig::default();
                    config.use_cache = false;

                    if let Some(ref o) = opts {
                        if let Ok(fmt) = o.get::<String>("format") {
                            config.output_format = match fmt.as_str() {
                                "plain" | "text" => xberg::core::config::OutputFormat::Plain,
                                "html" => xberg::core::config::OutputFormat::Html,
                                "json" => xberg::core::config::OutputFormat::Json,
                                _ => xberg::core::config::OutputFormat::Markdown,
                            };
                        }
                        if let Ok(ocr) = o.get::<bool>("ocr") {
                            if ocr {
                                config.force_ocr = true;
                                config.ocr = Some(xberg::core::config::OcrConfig::default());
                            }
                        }
                        if let Ok(secs) = o.get::<u64>("timeout_secs") {
                            config.extraction_timeout_secs = Some(secs);
                        }
                    }

                    let input = xberg::ExtractInput::from_uri(&path);
                    let result = match tokio::runtime::Handle::try_current() {
                        Ok(handle) => tokio::task::block_in_place(move || {
                            handle.block_on(xberg::extract(input, &config))
                        }),
                        Err(_) => {
                            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                                mlua::Error::external(format!("tokio runtime: {e}"))
                            })?;
                            rt.block_on(xberg::extract(input, &config))
                        }
                    }
                    .map_err(|e| mlua::Error::external(format!("xberg extract: {e}")))?;

                    let text: String = result
                        .results
                        .into_iter()
                        .map(|d| d.content)
                        .collect::<Vec<_>>()
                        .join("\n");
                    Ok(text)
                })?;
        xberg_tbl.set("extract", extract)?;

        // sift.ext.xberg.extract_bytes(bytes, mime, opts?) -> string
        let extract_bytes = self.lua.create_function(
            |_, (_ctx, bytes, mime_str, opts): (Table, mlua::String, String, Option<Table>)| {
                let mut config = xberg::ExtractionConfig::default();
                config.use_cache = false;

                if let Some(ref o) = opts {
                    if let Ok(fmt) = o.get::<String>("format") {
                        config.output_format = match fmt.as_str() {
                            "plain" | "text" => xberg::core::config::OutputFormat::Plain,
                            "html" => xberg::core::config::OutputFormat::Html,
                            "json" => xberg::core::config::OutputFormat::Json,
                            _ => xberg::core::config::OutputFormat::Markdown,
                        };
                    }
                    if let Ok(secs) = o.get::<u64>("timeout_secs") {
                        config.extraction_timeout_secs = Some(secs);
                    }
                }

                let raw: &[u8] = &bytes.as_bytes();
                let input = xberg::ExtractInput::from_bytes(raw.to_vec(), &mime_str, None);
                let result = match tokio::runtime::Handle::try_current() {
                    Ok(handle) => tokio::task::block_in_place(move || {
                        handle.block_on(xberg::extract(input, &config))
                    }),
                    Err(_) => {
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| mlua::Error::external(format!("tokio runtime: {e}")))?;
                        rt.block_on(xberg::extract(input, &config))
                    }
                }
                .map_err(|e| mlua::Error::external(format!("xberg extract: {e}")))?;

                let text: String = result
                    .results
                    .into_iter()
                    .map(|d| d.content)
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(text)
            },
        )?;
        xberg_tbl.set("extract_bytes", extract_bytes)?;

        // sift.ext.xberg.is_supported(mime) -> bool
        let is_supported = self
            .lua
            .create_function(|_, (_ctx, mime_str): (Table, String)| {
                Ok(xberg::core::mime::validate_mime_type(&mime_str).is_ok())
            })?;
        xberg_tbl.set("is_supported", is_supported)?;

        ext.set("xberg", xberg_tbl)?;
        Ok(())
    }

    /// Register `sift.ext.html` — HTML to Markdown conversion (gated behind `html-md` feature).
    #[cfg(feature = "html-md")]
    fn register_ext_html(&self, ext: &Table) -> Result<()> {
        let html_tbl = self.lua.create_table()?;

        // sift.ext.html.to_markdown(html, opts?) -> string
        let to_markdown =
            self.lua
                .create_function(|_, (_ctx, html, opts): (Table, String, Option<Table>)| {
                    let mut options = html_to_markdown_rs::ConversionOptions::default();
                    if let Some(ref o) = opts {
                        if let Ok(heading) = o.get::<String>("heading_style") {
                            options.heading_style = match heading.as_str() {
                                "underlined" => html_to_markdown_rs::HeadingStyle::Underlined,
                                "atx-closed" => html_to_markdown_rs::HeadingStyle::AtxClosed,
                                _ => html_to_markdown_rs::HeadingStyle::Atx,
                            };
                        }
                        if let Ok(link) = o.get::<String>("link_style") {
                            options.link_style = match link.as_str() {
                                "reference" => html_to_markdown_rs::LinkStyle::Reference,
                                _ => html_to_markdown_rs::LinkStyle::Inline,
                            };
                        }
                    }
                    let result = html_to_markdown_rs::convert(&html, Some(options))
                        .map_err(|e| mlua::Error::external(format!("html to markdown: {e}")))?;
                    Ok(result.content.unwrap_or_default())
                })?;
        html_tbl.set("to_markdown", to_markdown)?;

        ext.set("html", html_tbl)?;
        Ok(())
    }

    /// Register `sift.ext.markdown` — Markdown compression (gated behind `mdmin` feature).
    #[cfg(feature = "mdmin")]
    fn register_ext_markdown(&self, ext: &Table) -> Result<()> {
        let md_tbl = self.lua.create_table()?;

        // sift.ext.markdown.compress(md, opts?) -> string
        let compress =
            self.lua
                .create_function(|_, (_ctx, md, opts): (Table, String, Option<Table>)| {
                    let mut config = mdmin::Config::new(mdmin::Level::Medium);
                    if let Some(ref o) = opts {
                        if let Ok(level) = o.get::<i32>("level") {
                            config.level = match level {
                                0 => mdmin::Level::Off,
                                1 => mdmin::Level::Light,
                                3 => mdmin::Level::Structured,
                                4 => mdmin::Level::Ultra,
                                _ => mdmin::Level::Medium,
                            };
                        }
                        if let Ok(code) = o.get::<String>("code_blocks") {
                            config.code_blocks = match code.as_str() {
                                "preserve" => mdmin::CodeBlockMode::Preserve,
                                "compress-whitespace" => mdmin::CodeBlockMode::CompressWhitespace,
                                _ => mdmin::CodeBlockMode::Compress,
                            };
                        }
                        if let Ok(dict) = o.get::<bool>("dictionary") {
                            config.dictionary = dict;
                        }
                    }
                    let mut minifier = mdmin::Minifier::new(&config)
                        .map_err(|e| mlua::Error::external(format!("mdmin: {e}")))?;
                    let result = minifier
                        .minify(&md)
                        .map_err(|e| mlua::Error::external(format!("mdmin minify: {e}")))?;
                    Ok(result.output)
                })?;
        md_tbl.set("compress", compress)?;

        ext.set("markdown", md_tbl)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{SiftContext, SiftLua};
    use mlua::Table;
    use std::collections::HashMap;

    fn test_context() -> SiftContext {
        SiftContext {
            cwd: std::env::current_dir().unwrap(),
            cwd_str: std::env::current_dir().unwrap().display().to_string(),
            cmd_count: std::cell::Cell::new(0),
            env: HashMap::new(),
            session_id: None,
            raw_bytes: 0,
            filtered_bytes: 0,
        }
    }

    fn test_ctx(lua: &mlua::Lua) -> Table {
        let ctx = lua.create_table().unwrap();
        ctx.set("session_id", "test").unwrap();
        ctx.set("cmd_count", 0).unwrap();
        ctx.set("cwd", "/tmp").unwrap();
        ctx.set("command", "test").unwrap();
        ctx
    }

    #[test]
    fn test_mime_detect_pdf_by_extension() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let detect: mlua::Function = mime.get("detect").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = detect.call((ctx, "report.pdf")).unwrap();
        assert_eq!(result, "application/pdf");
    }

    #[test]
    fn test_mime_detect_png_by_extension() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let detect: mlua::Function = mime.get("detect").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = detect.call((ctx, "image.png")).unwrap();
        assert_eq!(result, "image/png");
    }

    #[test]
    fn test_mime_detect_jpeg_by_extension() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let detect: mlua::Function = mime.get("detect").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = detect.call((ctx, "photo.jpg")).unwrap();
        assert_eq!(result, "image/jpeg");
    }

    #[test]
    fn test_mime_detect_html_by_extension() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let detect: mlua::Function = mime.get("detect").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = detect.call((ctx, "page.html")).unwrap();
        assert_eq!(result, "text/html");
    }

    #[test]
    fn test_mime_detect_txt_by_extension() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let detect: mlua::Function = mime.get("detect").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = detect.call((ctx, "readme.txt")).unwrap();
        assert_eq!(result, "text/plain");
    }

    #[test]
    fn test_mime_detect_bytes_png() {
        // Minimal valid PNG header
        let png_bytes: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let detect_bytes: mlua::Function = mime.get("detect_bytes").unwrap();
        let ctx = test_ctx(&lua.lua);
        let lua_str = lua.lua.create_string(&png_bytes).unwrap();
        let result: String = detect_bytes.call((ctx, lua_str)).unwrap();
        assert_eq!(result, "image/png");
    }

    #[test]
    fn test_mime_extension_pdf() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let extension: mlua::Function = mime.get("extension").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = extension.call((ctx, "application/pdf")).unwrap();
        assert_eq!(result, "pdf");
    }

    #[test]
    fn test_mime_extension_jpeg() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let mime: Table = ext.get("mime").unwrap();
        let extension: mlua::Function = mime.get("extension").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = extension.call((ctx, "image/jpeg")).unwrap();
        // mime_guess may return "jfif" or "jpg" depending on version
        assert!(
            result == "jpg" || result == "jfif" || result == "jpeg",
            "expected jpg/jfif/jpeg, got {result}"
        );
    }

    #[test]
    fn test_xberg_is_nil_when_feature_disabled() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let xberg: mlua::Value = ext.get("xberg").unwrap_or(mlua::Value::Nil);
        #[cfg(not(feature = "xberg"))]
        assert!(
            matches!(xberg, mlua::Value::Nil),
            "xberg should be nil when feature disabled"
        );
        #[cfg(feature = "xberg")]
        assert!(
            !matches!(xberg, mlua::Value::Nil),
            "xberg should not be nil when feature enabled"
        );
    }

    #[cfg(feature = "xberg")]
    #[test]
    fn test_xberg_is_supported_pdf() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let xberg: Table = ext.get("xberg").unwrap();
        let is_supported: mlua::Function = xberg.get("is_supported").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: bool = is_supported.call((ctx, "application/pdf")).unwrap();
        assert!(result);
    }

    #[cfg(feature = "xberg")]
    #[test]
    fn test_xberg_is_supported_unknown() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let xberg: Table = ext.get("xberg").unwrap();
        let is_supported: mlua::Function = xberg.get("is_supported").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: bool = is_supported.call((ctx, "application/x-unknown")).unwrap();
        assert!(!result);
    }

    #[cfg(feature = "xberg")]
    #[test]
    fn test_xberg_extract_pdf() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let xberg: Table = ext.get("xberg").unwrap();
        let extract: mlua::Function = xberg.get("extract").unwrap();
        let ctx = test_ctx(&lua.lua);
        // Create a minimal valid PDF
        let pdf_path = std::env::temp_dir().join("test_xberg.pdf");
        let min_pdf = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Resources<<>>>>endobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n190\n%%EOF";
        std::fs::write(&pdf_path, min_pdf).unwrap();
        let result: String = extract
            .call((ctx, pdf_path.to_str().unwrap(), mlua::Value::Nil))
            .unwrap();
        let _ = std::fs::remove_file(&pdf_path);
        // Function should not error; content may be empty for minimal PDF
        assert!(
            result.len() < 10000,
            "xberg extract should not return huge output for minimal PDF"
        );
    }

    #[cfg(feature = "xberg")]
    #[test]
    fn test_xberg_extract_with_format_option() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let xberg: Table = ext.get("xberg").unwrap();
        let extract: mlua::Function = xberg.get("extract").unwrap();
        let ctx = test_ctx(&lua.lua);
        let pdf_path = std::env::temp_dir().join("test_xberg_fmt.pdf");
        let min_pdf = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Resources<<>>>>endobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n190\n%%EOF";
        std::fs::write(&pdf_path, min_pdf).unwrap();
        let opts = lua.lua.create_table().unwrap();
        opts.set("format", "plain").unwrap();
        let result: String = extract
            .call((ctx, pdf_path.to_str().unwrap(), opts))
            .unwrap();
        let _ = std::fs::remove_file(&pdf_path);
        // Function should not error; content may be empty for minimal PDF
        assert!(
            result.len() < 10000,
            "xberg extract should not return huge output for minimal PDF"
        );
    }

    #[cfg(feature = "html-md")]
    #[test]
    fn test_html_to_markdown_simple() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let html: Table = ext.get("html").unwrap();
        let to_markdown: mlua::Function = html.get("to_markdown").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: String = to_markdown
            .call((ctx, "<h1>Title</h1><p>Hello</p>", mlua::Value::Nil))
            .unwrap();
        assert!(result.contains("Title"), "should contain title text");
        assert!(result.contains("Hello"), "should contain paragraph text");
    }

    #[cfg(feature = "html-md")]
    #[test]
    fn test_html_to_markdown_with_heading_style() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let html: Table = ext.get("html").unwrap();
        let to_markdown: mlua::Function = html.get("to_markdown").unwrap();
        let ctx = test_ctx(&lua.lua);
        let opts = lua.lua.create_table().unwrap();
        opts.set("heading_style", "atx").unwrap();
        let result: String = to_markdown.call((ctx, "<h1>Title</h1>", opts)).unwrap();
        assert!(result.contains("#"), "atx style should use # for headings");
    }

    #[test]
    fn test_html_is_nil_when_feature_disabled() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let html_val: mlua::Value = ext.get("html").unwrap_or(mlua::Value::Nil);
        #[cfg(not(feature = "html-md"))]
        assert!(
            matches!(html_val, mlua::Value::Nil),
            "html should be nil when feature disabled"
        );
        #[cfg(feature = "html-md")]
        assert!(
            !matches!(html_val, mlua::Value::Nil),
            "html should not be nil when feature enabled"
        );
    }

    #[cfg(feature = "mdmin")]
    #[test]
    fn test_markdown_compress_level_2() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let md: Table = ext.get("markdown").unwrap();
        let compress: mlua::Function = md.get("compress").unwrap();
        let ctx = test_ctx(&lua.lua);
        let input = "# Title\n\nSome **bold** text\n";
        let opts = lua.lua.create_table().unwrap();
        opts.set("level", 2).unwrap();
        let result: String = compress.call((ctx, input, opts)).unwrap();
        assert!(!result.is_empty(), "compressed output should not be empty");
    }

    #[cfg(feature = "mdmin")]
    #[test]
    fn test_markdown_compress_level_0() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let md: Table = ext.get("markdown").unwrap();
        let compress: mlua::Function = md.get("compress").unwrap();
        let ctx = test_ctx(&lua.lua);
        let input = "hello";
        let opts = lua.lua.create_table().unwrap();
        opts.set("level", 0).unwrap();
        let result: String = compress.call((ctx, input, opts)).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_markdown_is_nil_when_feature_disabled() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let md_val: mlua::Value = ext.get("markdown").unwrap_or(mlua::Value::Nil);
        #[cfg(not(feature = "mdmin"))]
        assert!(
            matches!(md_val, mlua::Value::Nil),
            "markdown should be nil when feature disabled"
        );
        #[cfg(feature = "mdmin")]
        assert!(
            !matches!(md_val, mlua::Value::Nil),
            "markdown should not be nil when feature enabled"
        );
    }
}
