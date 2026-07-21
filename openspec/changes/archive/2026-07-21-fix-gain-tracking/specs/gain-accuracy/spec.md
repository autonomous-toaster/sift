# Gain Tracking Accuracy

## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Add stat call to head.lua |
| T1.2 | Add raw_bytes to head.lua unchanged returns |
| T1.3 | Add raw_bytes to head.lua handled return |
| T2.1 | Add stat call to tail.lua |
| T2.2 | Add raw_bytes to tail.lua unchanged returns |
| T2.3 | Add raw_bytes to tail.lua handled return |
| T3.1 | Add stat call to sed.lua |
| T3.2 | Add raw_bytes to sed.lua unchanged returns |
| T3.3 | Add raw_bytes to sed.lua handled return |
| T4.1 | Add raw_bytes to openspec.lua failure return |
| T4.2 | Add raw_bytes to openspec.lua success return |
| T5.1 | Verify head shows non-zero reduction |
| T5.2 | Verify tail shows non-zero reduction |
| T5.3 | Verify sed shows non-zero reduction |
| T5.4 | Verify openspec shows non-zero reduction |

## ADDED Requirements

### Requirement: head.lua gets file size

ALWAYS T1.1 SHALL add `sift.fs.stat()` call to get file size.

### Requirement: head.lua reports raw_bytes on all returns

ALWAYS T1.2 SHALL set `raw_bytes = stat.size` on unchanged returns.
ALWAYS T1.3 SHALL set `raw_bytes = stat.size` on handled return.

### Requirement: head.lua gain verified

T1.1 SHALL complete BEFORE T1.2 SHALL start.
T1.2 SHALL complete BEFORE T5.1 SHALL start.
T1.3 SHALL complete BEFORE T5.1 SHALL start.

#### Scenario: head gain is verified

- **WHEN** T1.1, T1.2, T1.3 complete
- **THEN** T5.1 SHALL confirm non-zero reduction in gain report

### Requirement: tail.lua gets file size

ALWAYS T2.1 SHALL add `sift.fs.stat()` call to get file size.

### Requirement: tail.lua reports raw_bytes on all returns

ALWAYS T2.2 SHALL set `raw_bytes = stat.size` on unchanged returns.
ALWAYS T2.3 SHALL set `raw_bytes = stat.size` on handled return.

### Requirement: tail.lua gain verified

T2.1 SHALL complete BEFORE T2.2 SHALL start.
T2.2 SHALL complete BEFORE T5.2 SHALL start.
T2.3 SHALL complete BEFORE T5.2 SHALL start.

#### Scenario: tail gain is verified

- **WHEN** T2.1, T2.2, T2.3 complete
- **THEN** T5.2 SHALL confirm non-zero reduction in gain report

### Requirement: sed.lua gets file size

ALWAYS T3.1 SHALL add `sift.fs.stat()` call to get file size.

### Requirement: sed.lua reports raw_bytes on all returns

ALWAYS T3.2 SHALL set `raw_bytes = stat.size` on unchanged returns.
ALWAYS T3.3 SHALL set `raw_bytes = stat.size` on handled return.

### Requirement: sed.lua gain verified

T3.1 SHALL complete BEFORE T3.2 SHALL start.
T3.2 SHALL complete BEFORE T5.3 SHALL start.
T3.3 SHALL complete BEFORE T5.3 SHALL start.

#### Scenario: sed gain is verified

- **WHEN** T3.1, T3.2, T3.3 complete
- **THEN** T5.3 SHALL confirm non-zero reduction in gain report

### Requirement: openspec.lua reports raw_bytes on failure

ALWAYS T4.1 SHALL set `raw_bytes = #(output .. stderr)` on failure.

### Requirement: openspec.lua reports raw_bytes on success

ALWAYS T4.2 SHALL set `raw_bytes = #output` on success.

### Requirement: openspec.lua gain verified

T4.1 SHALL complete BEFORE T5.4 SHALL start.
T4.2 SHALL complete BEFORE T5.4 SHALL start.

#### Scenario: openspec gain is verified

- **WHEN** T4.1 and T4.2 complete
- **THEN** T5.4 SHALL confirm non-zero reduction in gain report
