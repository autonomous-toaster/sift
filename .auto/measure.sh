#!/bin/bash
set -euo pipefail

# Pre-check: fast syntax check
cargo check --workspace --all-targets 2>&1 | tail -3

# Build
BUILD_START=$(date +%s%N)
cargo build --workspace 2>&1 | tail -3
BUILD_END=$(date +%s%N)
BUILD_µS=$(( (BUILD_END - BUILD_START) / 1000 ))

# Run throughput benchmark 3 times, report median
RUNS=3
THROUGHPUTS=()
COLD_STARTS=()
for i in $(seq 1 $RUNS); do
    OUTPUT=$(cargo bench -p sift --bench bench_throughput 2>&1 | grep "^METRIC")
    THROUGHPUT=$(echo "$OUTPUT" | grep "throughput_cps" | sed 's/.*=//')
    COLD_START=$(echo "$OUTPUT" | grep "cold_start_ns" | sed 's/.*=//')
    THROUGHPUTS+=("$THROUGHPUT")
    COLD_STARTS+=("$COLD_START")
done

# Sort and get median
IFS=$'\n' SORTED_T=($(sort -n <<<"${THROUGHPUTS[*]}"))
unset IFS
MEDIAN_T=${SORTED_T[$(( RUNS / 2 ))]}

IFS=$'\n' SORTED_C=($(sort -n <<<"${COLD_STARTS[*]}"))
unset IFS
MEDIAN_C=${SORTED_C[$(( RUNS / 2 ))]}

echo "METRIC throughput_cps=$MEDIAN_T"
echo "METRIC cold_start_ns=$MEDIAN_C"
echo "METRIC build_µs=$BUILD_µS"
