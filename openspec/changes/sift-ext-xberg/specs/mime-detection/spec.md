## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add infer + mime_guess deps, implement sift.ext.mime module |
| T3.1 | Verify MIME detection works for PDFs and common formats |

## ADDED Requirements

### Requirement: MIME detection from file path

T1.1 SHALL complete BEFORE T3.1 SHALL run.

#### Scenario: PDF detected by extension

- **WHEN** T1.1 SHALL complete
- **THEN** `sift.ext.mime.detect("report.pdf")` SHALL return `"application/pdf"`

#### Scenario: PNG detected by magic bytes

- **WHEN** T1.1 SHALL complete
- **THEN** `sift.ext.mime.detect_bytes(png_bytes)` SHALL return `"image/png"`

### Requirement: MIME detection from raw bytes

T1.1 SHALL complete BEFORE T3.1 SHALL run.

#### Scenario: JPEG detected from bytes

- **WHEN** T1.1 SHALL complete
- **THEN** `sift.ext.mime.detect_bytes(jpeg_bytes)` SHALL return `"image/jpeg"`

### Requirement: Extension reverse lookup

T1.1 SHALL complete BEFORE T3.1 SHALL run.

#### Scenario: PDF extension lookup

- **WHEN** T1.1 SHALL complete
- **THEN** `sift.ext.mime.extension("application/pdf")` SHALL return `"pdf"`
