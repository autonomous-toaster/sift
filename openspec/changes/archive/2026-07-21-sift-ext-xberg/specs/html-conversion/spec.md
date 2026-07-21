## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.3 | Add html-to-markdown-rs dep, implement sift.ext.html module |
| T1.5 | Register sift.ext table and all sub-modules in Lua API |

## ADDED Requirements

### Requirement: HTML to Markdown conversion

T1.3 SHALL complete BEFORE T1.5 SHALL run.

#### Scenario: Simple HTML converted to Markdown

- **WHEN** T1.3 SHALL complete
- **THEN** `sift.ext.html.to_markdown("<h1>Title</h1><p>Hello</p>")` SHALL return a string containing `# Title` and `Hello`

#### Scenario: Conversion with heading style option

- **WHEN** T1.3 SHALL complete
- **THEN** `sift.ext.html.to_markdown("<h1>Title</h1>", {heading_style="atx"})` SHALL return a string starting with `# `

### Requirement: Feature flag gating

T1.5 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: html available when feature enabled

- **WHEN** T1.5 SHALL complete AND the `html-md` feature SHALL be enabled
- **THEN** `sift.ext.html` SHALL NOT be nil

#### Scenario: html nil when feature disabled

- **WHEN** T1.5 SHALL complete AND the `html-md` feature SHALL NOT be enabled
- **THEN** `sift.ext.html` SHALL be nil
