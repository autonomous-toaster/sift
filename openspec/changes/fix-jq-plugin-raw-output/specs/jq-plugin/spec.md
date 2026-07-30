## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Fix `-r` output path to handle single-value and empty results |

## ADDED Requirements

### Requirement: jq-raw-output-handles-scalars

T1.1 SHALL complete. The jq plugin's `-r` output path SHALL ALWAYS handle both array results and scalar results.

When the result is a single value (number, string, boolean), it MUST be converted to a string and included in the output. When the result is an array, each element MUST be converted to a string, preserving existing behavior.

#### Scenario: single value from select filter

**WHEN** T1.1 runs
**THEN** `jq -r '.[] | select(.name == "x") | .id'` with matching input SHALL output the ID as a single line

#### Scenario: array from projection filter

**WHEN** T1.1 runs
**THEN** `jq -r '.[] | .name'` with array input SHALL output each name on a separate line
