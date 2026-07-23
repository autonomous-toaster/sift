# Gain Report Formatting

## Purpose

Enhance the `sift --gain` output with session scope, date range, and absolute savings information to make the report more informative without adding new CLI flags.

## Requirements

### Requirement: Gain report shows session scope

When no `AI_SESSION` is set, `sift --gain` SHALL display the number of unique sessions in the commands line (e.g., "15 (across 3 sessions)").

#### Scenario: All sessions mode shows session count

- **WHEN** `sift --gain` runs without `AI_SESSION`
- **THEN** the commands line SHALL include the session count

#### Scenario: Single session mode omits count

- **WHEN** `sift --gain` runs with `AI_SESSION` set
- **THEN** the commands line SHALL NOT include the session count

### Requirement: Gain report shows date range

`sift --gain` SHALL display the date range of recorded data as a "Period:" line (e.g., "Period: 2026-07-20 – 2026-07-22").

#### Scenario: Multi-day range

- **WHEN** data spans multiple days
- **THEN** `sift --gain` SHALL show "Period: YYYY-MM-DD – YYYY-MM-DD"

#### Scenario: Single-day range

- **WHEN** all data is from the same day
- **THEN** `sift --gain` SHALL show "Period: YYYY-MM-DD"

### Requirement: Gain report shows absolute savings

`sift --gain` SHALL display total tokens saved in KB alongside the percentage (e.g., "Reduction: 1.1% (114 bps, 5 KB saved)").

#### Scenario: Absolute savings displayed

- **WHEN** `sift --gain` runs
- **THEN** the reduction line SHALL include absolute KB saved
