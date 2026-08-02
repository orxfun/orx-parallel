#!/usr/bin/env bash
set -euo pipefail

# Central script to run all benchmark categories
# Usage: ./scripts/run_benches_all.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_BENCHES_SCRIPT="$SCRIPT_DIR/run_benches_of.sh"

if [[ ! -f "$RUN_BENCHES_SCRIPT" ]]; then
    echo "Error: $RUN_BENCHES_SCRIPT not found"
    exit 1
fi

# Benchmark categories to run (exclude xap, xap_pll)
CATEGORIES=(
    # "arbitrary_iter"
    # "collect"
    # "contention_merge"
    # "early_exit"
    # "fallible"
    # "first"
    "het"
    "memory_pressure"
    "recursive"
    "reduce"
    "stateful_using"
    "throughput_linear"
    "xap"
    "xap_pll"
)

echo "Running all benchmark categories..."
echo ""

for category in "${CATEGORIES[@]}"; do
    echo "================================"
    echo "Running: $category"
    echo "================================"
    "$RUN_BENCHES_SCRIPT" "$category" || {
        echo "Warning: Benchmarks for category '$category' failed or timed out"
    }
    echo ""
done

echo "All benchmark categories completed!"
