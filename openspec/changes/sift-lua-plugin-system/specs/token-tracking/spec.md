# Token Tracking

## Task Reference

| Task ID | Description |
|---------|-------------|
| T5.1 | Add token tracking columns to session store schema |
| T5.2 | Compute and store per-command metrics after plugin execution |
| T5.3 | Generate bypass notices based on tracking data |

## Requirements

### Requirement: Schema update precedes metric storage

T5.1 SHALL complete BEFORE T5.2 SHALL run.

#### Scenario: Columns exist before writes

- **WHEN** T5.1 adds columns to the session store
- **THEN** T5.2 SHALL write metrics to those columns.

### Requirement: Metric storage precedes notice generation

T5.2 SHALL complete BEFORE T5.3 SHALL run.

#### Scenario: Notices use stored metrics

- **WHEN** T5.2 stores metrics for a command
- **THEN** T5.3 SHALL use those metrics to generate bypass notices.

### Requirement: Per-command metrics are stored

ALWAYS T5.2 SHALL store `raw_bytes`, `filtered_bytes`, `reduction_pct`, `plugin_name`, and `status` for every command.

#### Scenario: Metrics recorded

- **WHEN** a plugin returns a result
- **THEN** T5.2 SHALL compute and store the metrics in the session DB.

### Requirement: Bypass notice for unchanged output

ALWAYS T5.3 SHALL append a bypass notice when a plugin returns "unchanged".

#### Scenario: Unchanged notice

- **WHEN** a plugin returns `{status="unchanged", message="[sift] foo.rs unchanged"}`
- **THEN** T5.3 SHALL append `[sift] Use 'command cat /path/to/foo.rs' for full content`.

### Requirement: Bypass notice for truncated output

ALWAYS T5.3 SHALL append a bypass notice with full output path when a plugin returns "truncated".

#### Scenario: Truncated notice

- **WHEN** a plugin returns `{status="truncated", full_output_path="/tmp/sift/.../log"}`
- **THEN** T5.3 SHALL append `[sift] Full output: /tmp/sift/.../log` and `[sift] Use 'command cat /tmp/sift/.../log' for raw output`.
