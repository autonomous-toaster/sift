## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add session count, date range, and absolute savings fields to GainReport |
| T1.2 | Track session count and date range during iteration in generate_gain_report |
| T1.3 | Render new fields in format_gain_report output |

## ADDED Requirements

### Requirement: Gain report shows session scope

T1.2 SHALL complete BEFORE T1.3 SHALL run. When no `AI_SESSION` is set, `sift --gain` SHALL display the number of unique sessions in the commands line (e.g., "15 (across 3 sessions)").

#### Scenario: All sessions mode shows session count

- **WHEN** T1.1 and T1.2 complete
- **THEN** `sift --gain` without `AI_SESSION` SHALL show session count in the commands line

#### Scenario: Single session mode omits count

- **WHEN** T1.1 and T1.2 complete
- **THEN** `sift --gain` with `AI_SESSION` set SHALL NOT show session count

### Requirement: Gain report shows date range

T1.2 SHALL complete BEFORE T1.3 SHALL run. `sift --gain` SHALL display the date range of recorded data as a "Period:" line (e.g., "Period: 2026-07-20 – 2026-07-22").

#### Scenario: Multi-day range

- **WHEN** T1.1 and T1.2 complete
- **THEN** `sift --gain` SHALL show "Period: YYYY-MM-DD – YYYY-MM-DD" when data spans multiple days

#### Scenario: Single-day range

- **WHEN** T1.1 and T1.2 complete
- **THEN** `sift --gain` SHALL show "Period: YYYY-MM-DD" when all data is from the same day

### Requirement: Gain report shows absolute savings

T1.1 SHALL complete BEFORE T1.3 SHALL run. `sift --gain` SHALL display total tokens saved in KB alongside the percentage (e.g., "Reduction: 1.1% (114 bps, 5 KB saved)").

#### Scenario: Absolute savings displayed

- **WHEN** T1.1 and T1.3 complete
- **THEN** `sift --gain` SHALL show absolute savings in KB
