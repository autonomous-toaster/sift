# Dedup protection for repeated unchanged responses

## ADDED

### Recent unchanged tracking in SiftLua

- Add `Mutex<Vec<(String, u128)>>` field to `SiftLua` for tracking recent command+status pairs with timestamps
- In `dispatch()`, when status is "unchanged", track the command+message key with current timestamp
- Prune entries older than 10 seconds before each check
- If the same key appears 3+ times within the 10-second window, append a stronger hint on a new line
- Keep a sliding window of the last 10 entries

## Verification

- Running the same cached command 3+ times within 10s adds a stronger hint
- Different commands don't interfere with each other's counters
- Entries older than 10s are pruned (no false positives from spaced-out reads)
- Sliding window prevents unbounded memory growth
