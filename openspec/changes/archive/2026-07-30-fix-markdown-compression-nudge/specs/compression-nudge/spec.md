## ADDED Requirements

### Requirement: Cat plugin SHALL nudge on markdown compression
When `cat.lua` compresses a `.md` or `.markdown` file via mdmin, it SHALL emit a nudge via `sift.nudge()` with the path to the raw file using `command cat`.

#### Scenario: cat reads markdown file
- **WHEN** T1.1 reads a `.md` file through the cat plugin and the content is compressed via mdmin
- **THEN** the output SHALL include a nudge `[sift] raw: 'command cat <path>'`

### Requirement: Sift-read plugin SHALL nudge on content compression
When `sift-read.lua` compresses content (via xberg or mdmin), it SHALL emit a nudge via `sift.nudge()` with the path to the raw file using `command cat`.

#### Scenario: sift-read reads markdown file
- **WHEN** T2.1 reads a `.md` file through the sift-read plugin and the content is compressed
- **THEN** the output SHALL include a nudge `[sift] raw: 'command cat <path>'`
