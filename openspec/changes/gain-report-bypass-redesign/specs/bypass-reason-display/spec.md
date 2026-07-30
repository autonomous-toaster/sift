## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Accept and store real plugin name in `handle_passthrough_status` |
| T1.2 | Store actual command name in passthrough conversation entries |
| T2.1 | Extract command name from all conversation entry types |
| T2.2 | Compute bypass ratio and reason per command |
| T2.3 | Change BypassEntry struct: replace cache_hits/hit_rate with total_calls/reason |
| T2.4 | Aggregate `__default__` entries by command name into UnmatchedEntry list |
| T3.1 | Update "Top bypassed" rendering to show ratio and reason |
| T3.2 | Add "Commands without plugins" section to rendering |

## ADDED Requirements

### Requirement: passthrough-recording-accuracy

T1.1 SHALL complete BEFORE T1.2 SHALL store actual command name.

The `handle_passthrough_status` function records plugin name and command for every passthrough entry. The plugin name MUST be the actual plugin that returned passthrough, not a hardcoded string. The command field MUST store the actual command name the user ran, not the first word after shell splitting.

#### Scenario: explicit bypass preserves command name

**WHEN** T1.1 and T1.2 complete
**THEN** `command head -n 10` SHALL record a conversation entry with `command = "head"` and `plugin_name = "command"`

#### Scenario: plugin passthrough preserves plugin name

**WHEN** T1.1 and T1.2 complete
**THEN** `head -n 10` with piped stdin SHALL record a conversation entry with `command = "head"` and `plugin_name = "head"`

### Requirement: bypass-ratio-display

T2.1 SHALL complete BEFORE T2.2 SHALL compute ratio. T2.2 SHALL complete BEFORE T2.3 SHALL change struct. T2.3 SHALL complete BEFORE T3.1 SHALL update rendering.

The "Top bypassed" section MUST show bypass count, total calls, bypass percentage, and bypass reason for each command. The cache hit rate column MUST be removed.

#### Scenario: bypass section shows ratio

**WHEN** T3.1 runs
**THEN** the "Top bypassed" section SHALL display lines in the format `head 802/830 (97%) passthrough`

### Requirement: unmatched-commands-section

T2.4 SHALL complete BEFORE T3.2 SHALL add section.

A new "Commands without plugins" section MUST show `__default__` entries aggregated by command name, ordered by call count descending, limited to the top 10 entries.

#### Scenario: unmatched section shows top commands

**WHEN** T3.2 runs
**THEN** the "Commands without plugins" section SHALL display lines in the format `git 200 calls`
