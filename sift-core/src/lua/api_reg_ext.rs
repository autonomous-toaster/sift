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
        #[cfg(feature = "scred")]
        self.register_ext_scred(&ext)?;
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
                    let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        tokio::task::block_in_place(move || {
                            handle.block_on(xberg::extract(input, &config))
                        })
                    } else {
                        let rt = tokio::runtime::Runtime::new()
                            .map_err(|e| mlua::Error::external(format!("tokio runtime: {e}")))?;
                        rt.block_on(xberg::extract(input, &config))
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
                let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    tokio::task::block_in_place(move || {
                        handle.block_on(xberg::extract(input, &config))
                    })
                } else {
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| mlua::Error::external(format!("tokio runtime: {e}")))?;
                    rt.block_on(xberg::extract(input, &config))
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

    /// Register `sift.ext.scred` — secret redaction (gated behind `scred` feature).
    #[cfg(feature = "scred")]
    fn register_ext_scred(&self, ext: &Table) -> Result<()> {
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};

        use scred_redactor::{
            redactor::{RedactionConfig, RedactionEngine},
            streaming::RedactionStream,
        };
        use scred_redactor::scred_detector::{SIMPLE_PREFIX_PATTERNS, PREFIX_VALIDATION_PATTERNS};

        let scred_tbl = self.lua.create_table()?;
        let lua = self.lua.clone();

        // Build name-to-pattern-type-ID mapping at registration time
        let mut name_to_id: HashMap<&'static str, u16> = HashMap::new();
        for (i, p) in SIMPLE_PREFIX_PATTERNS.iter().enumerate() {
            name_to_id.insert(p.name, i as u16);
        }
        for (i, p) in PREFIX_VALIDATION_PATTERNS.iter().enumerate() {
            name_to_id.insert(p.name, 100u16 + i as u16);
        }

        // Shared engine for all streams
        let engine = Arc::new(RedactionEngine::new(RedactionConfig::default()));

        // sift.ext.scred.create_transform(opts?) -> (transform_fn, finalize_fn)
        let engine_ct = engine.clone();
        let name_to_id_ct = name_to_id.clone();
        let lua_ct = lua.clone();
        let create_transform = lua.clone().create_function(
            move |_, (_ctx, opts): (Table, Option<Table>)| {
                let allowed = parse_allowed_patterns(opts.as_ref(), &name_to_id_ct)
                    .map_err(|e: anyhow::Error| mlua::Error::external(e.to_string()))?;
                let stream = Arc::new(Mutex::new(
                    RedactionStream::with_allowed_patterns(engine_ct.clone(), allowed)
                ));

                // Feed function — called from background thread per chunk
                let feed_stream = stream.clone();
                let feed_fn = lua_ct.create_function(move |_, chunk: String| {
                    let result = match feed_stream.lock() {
                        Ok(mut s) => s.feed(chunk.as_bytes()),
                        Err(_) => chunk.into_bytes(),  // poisoned → passthrough
                    };
                    Ok(String::from_utf8_lossy(&result).to_string())
                })?;

                // Finalize function — called by plugin after exec completes
                let lua_final = lua_ct.clone();
                let finalize_fn = lua_ct.create_function(move |_, ()| {
                    let mut s = match stream.lock() {
                        Ok(s) => s,
                        Err(_) => {
                            return Ok((String::new(), mlua::Value::Nil));
                        }
                    };
                    let (final_chunk, stats) = s.finalize();
                    let stats_table = lua_final.create_table()?;
                    stats_table.set("bytes_read", stats.bytes_read)?;
                    stats_table.set("bytes_written", stats.bytes_written)?;
                    stats_table.set("chunks_processed", stats.chunks_processed)?;
                    stats_table.set("patterns_found", stats.patterns_found)?;
                    stats_table.set("errors", stats.errors)?;
                    Ok((String::from_utf8_lossy(&final_chunk).to_string(), mlua::Value::Table(stats_table)))
                })?;

                Ok((feed_fn, finalize_fn))
            },
        )?;
        scred_tbl.set("create_transform", create_transform)?;

        // sift.ext.scred.redact(text, opts?) -> string
        let engine_redact = engine.clone();
        let name_to_id_redact = name_to_id.clone();
        let redact_fn = lua.clone().create_function(
            move |_, (_ctx, text, opts): (Table, String, Option<Table>)| {
                let allowed = parse_allowed_patterns(opts.as_ref(), &name_to_id_redact)
                    .map_err(|e: anyhow::Error| mlua::Error::external(e.to_string()))?;
                let mut stream = RedactionStream::with_allowed_patterns(
                    engine_redact.clone(), allowed
                );
                let _ = stream.feed(text.as_bytes());
                let (final_chunk, _stats) = stream.finalize();
                Ok(String::from_utf8_lossy(&final_chunk).to_string())
            },
        )?;
        scred_tbl.set("redact", redact_fn)?;

        ext.set("scred", scred_tbl)?;
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
                        if let Ok(strip) = o.get::<bool>("strip_svg") {
                            if strip {
                                options.exclude_selectors = vec!["svg".into(), "math".into()];
                            }
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

/// Parse redact opts into a set of allowed pattern type IDs.
/// Empty set = all patterns allowed.
#[cfg(feature = "scred")]
fn parse_allowed_patterns(
    opts: Option<&mlua::Table>,
    name_to_id: &std::collections::HashMap<&'static str, u16>,
) -> anyhow::Result<std::collections::HashSet<u16>> {
    use scred_redactor::pattern_selector::PatternFilter;

    let Some(ref o) = opts else { return Ok(std::collections::HashSet::new()); };
    let Ok(redact_str) = o.get::<String>("redact") else { return Ok(std::collections::HashSet::new()); };
    let redact_str = redact_str.trim();
    if redact_str.is_empty() || redact_str.eq_ignore_ascii_case("ALL") {
        return Ok(std::collections::HashSet::new());
    }
    let mut allowed = std::collections::HashSet::new();
    for part in redact_str.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        // Try exact name match
        if let Some(&id) = name_to_id.get(part) {
            allowed.insert(id);
            continue;
        }
        // Try glob match against pattern names
        let matcher = PatternFilter::GlobName(part.to_string());
        for (&name, &id) in name_to_id {
            if matcher.matches(name, scred_redactor::metadata_cache::RiskTier::Critical) {
                allowed.insert(id);
            }
        }
    }
    Ok(allowed)
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
    fn test_html_to_markdown_strip_svg() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let html: Table = ext.get("html").unwrap();
        let to_markdown: mlua::Function = html.get("to_markdown").unwrap();
        let ctx = test_ctx(&lua.lua);

        // strip_svg=true should remove SVG elements
        let opts = lua.lua.create_table().unwrap();
        opts.set("strip_svg", true).unwrap();
        let html_with_svg = "<p>Hello</p><svg><path d=\"M10 10\"/></svg><p>World</p>";
        let result: String = to_markdown
            .call((ctx.clone(), html_with_svg, opts))
            .unwrap();
        assert!(result.contains("Hello"), "should keep text before SVG");
        assert!(result.contains("World"), "should keep text after SVG");
        assert!(
            !result.contains("data:image/svg+xml;base64"),
            "should not contain base64 SVG data URI"
        );

        // strip_svg=false (default) should preserve SVG as base64
        let result2: String = to_markdown
            .call((ctx, html_with_svg, mlua::Value::Nil))
            .unwrap();
        assert!(
            result2.contains("data:image/svg+xml;base64"),
            "default should contain base64 SVG data URI"
        );
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

    #[cfg(feature = "scred")]
    #[test]
    fn test_scred_is_nil_when_feature_disabled() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let scred_val: mlua::Value = ext.get("scred").unwrap_or(mlua::Value::Nil);
        #[cfg(not(feature = "scred"))]
        assert!(
            matches!(scred_val, mlua::Value::Nil),
            "scred should be nil when feature disabled"
        );
        #[cfg(feature = "scred")]
        assert!(
            !matches!(scred_val, mlua::Value::Nil),
            "scred should not be nil when feature enabled"
        );
    }

    #[cfg(feature = "scred")]
    #[test]
    fn test_scred_redact_redacts_aws_key() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let scred: Table = ext.get("scred").unwrap();
        let redact: mlua::Function = scred.get("redact").unwrap();
        let ctx = test_ctx(&lua.lua);
        let input = "AKIAIOSFODNN7EXAMPLE";
        let result: String = redact.call((ctx, input, mlua::Value::Nil)).unwrap();
        assert_eq!(result.len(), input.len(), "redacted output should be same length");
        assert_ne!(result, input, "AWS key should be redacted");
        assert!(
            result.starts_with("AKIA"),
            "redacted AWS key should preserve prefix, got: {result}"
        );
    }

    #[cfg(feature = "scred")]
    #[test]
    fn test_scred_redact_passthrough_plain_text() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let scred: Table = ext.get("scred").unwrap();
        let redact: mlua::Function = scred.get("redact").unwrap();
        let ctx = test_ctx(&lua.lua);
        let input = "hello world this is plain text";
        let result: String = redact.call((ctx, input, mlua::Value::Nil)).unwrap();
        assert_eq!(result, input, "plain text should pass through unchanged");
    }

    #[cfg(feature = "scred")]
    #[test]
    fn test_scred_create_transform_returns_functions() {
        let lua = SiftLua::new(None, test_context()).unwrap();
        let sift: Table = lua.lua.globals().get("sift").unwrap();
        let ext: Table = sift.get("ext").unwrap();
        let scred: Table = ext.get("scred").unwrap();
        let create_transform: mlua::Function = scred.get("create_transform").unwrap();
        let ctx = test_ctx(&lua.lua);
        let result: mlua::MultiValue = create_transform.call((ctx, mlua::Value::Nil)).unwrap();
        let results: Vec<mlua::Value> = result.into_iter().collect();
        assert_eq!(results.len(), 2, "create_transform should return 2 functions");
        assert!(
            matches!(&results[0], mlua::Value::Function(_)),
            "first return value should be a function"
        );
        assert!(
            matches!(&results[1], mlua::Value::Function(_)),
            "second return value should be a function"
        );
    }
}
