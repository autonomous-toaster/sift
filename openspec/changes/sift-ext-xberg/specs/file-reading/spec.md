## Task Reference

| Task ID | Description |
|---------|-------------|
| T2.1 | Modify sift-read.lua to detect MIME and route to xberg |
| T3.3 | Verify sift-read routes PDFs to xberg and caches extracted text |

## MODIFIED Requirements

### Requirement: Binary document detection and extraction

T2.1 SHALL complete BEFORE T3.3 SHALL run.

sift-read SHALL detect binary document formats via MIME type detection. When the detected MIME type is supported by xberg and the `xberg` feature is enabled, sift-read SHALL extract text via `sift.ext.xberg.extract()` instead of reading raw bytes. The extracted text SHALL be cached by the SHA256 hash of the original file bytes. When xberg is unavailable or the MIME type is unsupported, sift-read SHALL fall back to `sift.fs.read()`.

#### Scenario: PDF routed to xberg

- **WHEN** T2.1 SHALL complete AND the file SHALL be a PDF AND the `xberg` feature SHALL be enabled
- **THEN** sift-read SHALL call `sift.ext.xberg.extract()` and SHALL return extracted Markdown text

#### Scenario: PDF cached on second read

- **WHEN** T2.1 SHALL complete AND a PDF SHALL have been read once AND the same PDF SHALL be read again
- **THEN** sift-read SHALL return `"unchanged"` with a cached marker

#### Scenario: Text file unchanged behavior

- **WHEN** T2.1 SHALL complete AND the file SHALL be a plain text file
- **THEN** sift-read SHALL read via `sift.fs.read()` as before (no behavioral change)

#### Scenario: xberg unavailable falls back to raw read

- **WHEN** T2.1 SHALL complete AND the file SHALL be a PDF AND the `xberg` feature SHALL NOT be enabled
- **THEN** sift-read SHALL fall back to `sift.fs.read()` and SHALL return raw binary content
