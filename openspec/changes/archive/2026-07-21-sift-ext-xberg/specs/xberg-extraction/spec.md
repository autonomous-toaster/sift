## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.2 | Add xberg dep with pdf feature, implement sift.ext.xberg module |
| T1.5 | Register sift.ext table and all sub-modules in Lua API |
| T3.2 | Verify xberg extraction produces correct Markdown output |

## ADDED Requirements

### Requirement: Document-to-text extraction from file path

T1.2 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: PDF extracted to Markdown

- **WHEN** T1.2 SHALL complete
- **THEN** `sift.ext.xberg.extract("report.pdf", {format="markdown"})` SHALL return a non-empty string containing Markdown

#### Scenario: Extraction with page range

- **WHEN** T1.2 SHALL complete
- **THEN** `sift.ext.xberg.extract("report.pdf", {pages={1, "3-5"}})` SHALL return text only from pages 1, 3, 4, and 5

### Requirement: Extraction from raw bytes

T1.2 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: PDF bytes extracted to text

- **WHEN** T1.2 SHALL complete
- **THEN** `sift.ext.xberg.extract_bytes(pdf_bytes, "application/pdf", {format="plain"})` SHALL return a non-empty string

### Requirement: Supported format detection

T1.2 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: PDF is supported

- **WHEN** T1.2 SHALL complete
- **THEN** `sift.ext.xberg.is_supported("application/pdf")` SHALL return `true`

#### Scenario: Unknown format is not supported

- **WHEN** T1.2 SHALL complete
- **THEN** `sift.ext.xberg.is_supported("application/x-unknown")` SHALL return `false`

### Requirement: Feature flag gating

T1.5 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: xberg available when feature enabled

- **WHEN** T1.5 SHALL complete AND the `xberg` feature SHALL be enabled
- **THEN** `sift.ext.xberg` SHALL NOT be nil

#### Scenario: xberg nil when feature disabled

- **WHEN** T1.5 SHALL complete AND the `xberg` feature SHALL NOT be enabled
- **THEN** `sift.ext.xberg` SHALL be nil
