#!/bin/bash
set -euo pipefail

# Pre-check: fast syntax check
cargo check --workspace --all-targets 2>&1 | tail -5

# Build first (required for tests)
BUILD_START=$(date +%s%N)
cargo build --workspace 2>&1 | tail -3
BUILD_END=$(date +%s%N)
BUILD_µS=$(( (BUILD_END - BUILD_START) / 1000 ))

# Run tests 5 times, report median
RUNS=5
TIMES=()
for i in $(seq 1 $RUNS); do
    START=$(date +%s%N)
    cargo test --workspace 2>&1 | tail -1
    END=$(date +%s%N)
    ELAPSED=$(( (END - START) / 1000 ))
    TIMES+=("$ELAPSED")
done

# Sort and get median
IFS=$'\n' SORTED=($(sort -n <<<"${TIMES[*]}"))
unset IFS
MEDIAN=${SORTED[$(( RUNS / 2 ))]}

echo "METRIC test_µs=$MEDIAN"
echo "METRIC build_µs=$BUILD_µS"
