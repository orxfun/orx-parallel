#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/run_benches_of.sh <category>
# Example: ./scripts/run_benches_of.sh recursive

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <category>"
    echo "Example: $0 recursive"
    exit 1
fi

CATEGORY="$1"
BENCH_DIR="benches/$CATEGORY"
RESULTS_DIR="docs/bench-ui/results/$CATEGORY"

# Validate that the category directory exists
if [[ ! -d "$BENCH_DIR" ]]; then
    echo "Error: Directory $BENCH_DIR does not exist"
    exit 1
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Extract all benchmark names from Cargo.toml that have the path "benches/$CATEGORY/*"
# We look for [[bench]] sections with path = "benches/$CATEGORY/..."
BENCHMARKS=$(awk -v pattern="benches/$CATEGORY/" '
    /^\[\[bench\]\]/ {
        if (bench_name != "" && index(bench_path, pattern) > 0) {
            print bench_name
        }
        in_bench = 1
        bench_name = ""
        bench_path = ""
        next
    }
    in_bench && /^name = / {
        match($0, /"([^"]+)"/, m)
        bench_name = m[1]
    }
    in_bench && /^path = / {
        match($0, /"([^"]+)"/, m)
        bench_path = m[1]
    }
    END {
        if (bench_name != "" && index(bench_path, pattern) > 0) {
            print bench_name
        }
    }
' Cargo.toml)

if [[ -z "$BENCHMARKS" ]]; then
    echo "Error: No benchmarks found for category '$CATEGORY' in Cargo.toml"
    exit 1
fi

echo "Running benchmarks for category: $CATEGORY"
echo "Results will be stored in: $RESULTS_DIR"
echo ""

# For each benchmark, run it and copy the summary CSV
while read -r bench_name; do
    echo "Running: $bench_name"
    
    # Run the benchmark
    cargo bench --bench "$bench_name"
    
    # Criterion stores this benchmark under the full bench name with underscores.
    criterion_dir="${bench_name//-/_}"
    criterion_path="target/criterion/$criterion_dir/summary_$criterion_dir.csv"

    # Benchmark names can use hyphens while category directories use underscores.
    # We strip the category prefix only for the output file name under bench-ui/results.
    bench_stem="$bench_name"
    bench_stem="${bench_stem#${CATEGORY}_}"
    bench_stem="${bench_stem#${CATEGORY//_/-}-}"
    
    # Save the CSV under the bench stem so bench-ui can discover it without the category prefix.
    result_file="${bench_stem//-/_}"
    result_file="$result_file.csv"
    
    # Copy the summary CSV if it exists
    if [[ -f "$criterion_path" ]]; then
        cp "$criterion_path" "$RESULTS_DIR/$result_file"
        echo "  → Saved to: $RESULTS_DIR/$result_file"
    else
        echo "  ⚠ Warning: Summary CSV not found at $criterion_path"
    fi
    
    echo ""
done <<< "$BENCHMARKS"

echo "Updating bench-ui catalog manifest..."
python3 scripts/update_bench_ui_catalog.py

echo "All benchmarks for category '$CATEGORY' completed."
