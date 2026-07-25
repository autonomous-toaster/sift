# Gain Enhancements

## Purpose

Strengthen the `--gain` report with absolute byte/token savings, per-command breakdown, sequential duplicate detection, and time-series data.

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add `command`, `exec_time_ms`, `cache_hit` columns to `conversation_cache` table |
| T1.2 | Add fields to `ConversationEntry` struct |
| T1.3 | Update `record_conversation()` to accept and store new fields |
| T2.1 | Pass command string to `record_conversation()` |
| T2.2 | Record execution time during dispatch |
| T2.3 | Set `cache_hit` flag when output_format is "unchanged" |
| T3.1 | Add `total_saved_bytes`, token estimate fields to `GainReport` |
| T3.2 | Add `BypassEntry` struct and aggregate top N bypassed commands |
| T3.3 | Add `SequentialDup` struct and detect sequential duplicates |
| T3.4 | Add `DayEntry` struct and aggregate time-series data |
| T4.1 | Update `format_gain_report()` to show absolute bytes and tokens |
| T4.2 | Add per-command breakdown section to report |
| T4.3 | Add sequential duplicate section (verbose mode only) |
| T4.4 | Add time-series section with ASCII sparklines |
| T5.1 | Add `--daily` and `--weekly` CLI flags |
| T5.2 | Wire CLI flags to gain report filtering |
| T5.3 | Run `cargo test` to verify no regressions |

## Requirements

### Requirement: conversation_cache schema is extended

T1.1 SHALL complete BEFORE T1.2 SHALL run.
T1.2 SHALL complete BEFORE T1.3 SHALL run.

#### Scenario: New columns are added

- **WHEN** T1.1 runs
- **THEN** the `conversation_cache` table SHALL have `command TEXT`, `exec_time_ms INTEGER`, and `cache_hit INTEGER` columns

- **WHEN** T1.2 runs
- **THEN** the `ConversationEntry` struct SHALL have `command`, `exec_time_ms`, and `cache_hit` fields

- **WHEN** T1.3 runs
- **THEN** `record_conversation()` SHALL accept and store the new fields

### Requirement: Dispatch records command metadata

T2.1 SHALL complete BEFORE T2.2 SHALL run.
T2.2 SHALL complete BEFORE T2.3 SHALL run.

#### Scenario: Command metadata is recorded

- **WHEN** T2.1 runs
- **THEN** the command string SHALL be recorded during dispatch, truncated to 200 characters after `cd` prefix peeling

- **WHEN** T2.2 runs
- **THEN** execution time SHALL be measured and recorded

- **WHEN** T2.3 runs
- **THEN** `cache_hit` SHALL be set to `true` when `output_format` is `"unchanged"`

### Requirement: Gain report aggregation is sequential

T3.1 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: Gain report fields are added

- **WHEN** T3.1 runs
- **THEN** `GainReport` SHALL include `total_saved_bytes`, `total_saved_tokens`, `total_raw_tokens`, and `total_filtered_tokens`

- **WHEN** T3.2 runs
- **THEN** the top N most-bypassed commands SHALL be aggregated into `BypassEntry` entries

### Requirement: Bypass and time-series aggregation are concurrent

T3.2 SHALL complete CONCURRENTLY with T3.3.
T3.3 SHALL complete CONCURRENTLY with T3.4.

#### Scenario: Concurrent aggregation

- **WHEN** T3.2, T3.3, and T3.4 run concurrently
- **THEN** bypassed commands, sequential duplicates, and time-series data SHALL all be aggregated

### Requirement: Gain report display is enhanced

T4.1 SHALL complete BEFORE T4.2 SHALL run.
T4.2 SHALL complete BEFORE T4.3 SHALL run.
T4.3 SHALL complete BEFORE T4.4 SHALL run.

#### Scenario: Display is enhanced

- **WHEN** T4.1 runs
- **THEN** the report SHALL display absolute bytes and estimated token counts alongside percentages

- **WHEN** T4.2 runs
- **THEN** the report SHALL include a "Top bypassed" section

- **WHEN** T4.3 runs
- **THEN** the report SHALL include a "Sequential duplicates" section in verbose mode

- **WHEN** T4.4 runs
- **THEN** the report SHALL include a time-series section with ASCII sparklines

### Requirement: CLI flags are added

T5.1 SHALL complete BEFORE T5.2 SHALL run.
T5.2 SHALL complete BEFORE T5.3 SHALL run.

#### Scenario: CLI flags are added

- **WHEN** T5.1 runs
- **THEN** `--daily` and `--weekly` flags SHALL be added to the CLI

- **WHEN** T5.2 runs
- **THEN** the flags SHALL filter the time-series data in the gain report

- **WHEN** T5.3 runs
- **THEN** `cargo test` SHALL pass with no regressions
