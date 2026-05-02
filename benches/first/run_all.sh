#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_TOML="$(cd "$SCRIPT_DIR/../.." && pwd)/Cargo.toml"
RESULTS_FILE="$SCRIPT_DIR/run_results.txt"

: > "$RESULTS_FILE"

benchmark_files=("$SCRIPT_DIR"/*.rs)
total_benchmarks=0
for benchmark_file in "${benchmark_files[@]}"; do
    [ -e "$benchmark_file" ] || continue
    total_benchmarks=$((total_benchmarks + 1))
done

benchmark_index=0

for benchmark_file in "${benchmark_files[@]}"; do
    [ -e "$benchmark_file" ] || continue

    benchmark_index=$((benchmark_index + 1))

    benchmark_basename="$(basename "$benchmark_file")"

    benchmark_name="$({ sed -n 's/.*Exp\.bench(c, "\([^"]*\)",.*/\1/p' "$benchmark_file"; } | head -n 1)"

    if [[ -z "$benchmark_name" ]]; then
        printf 'could not parse benchmark name from %s\n' "$benchmark_file" >&2
        continue
    fi

    sed -i '/^\[\[bench\]\]$/,/^\[/ s|^path = "benches/first/.*\.rs"$|path = "benches/first/'"$benchmark_basename"'"|' "$CARGO_TOML"

    printf '[%d / %d] %s\n' "$benchmark_index" "$total_benchmarks" "$benchmark_name"

    tmp_output=$(mktemp)
    tmp_clean=$(mktemp)
    tmp_extract=$(mktemp)
    (cd "$SCRIPT_DIR/../.." && cargo bench --color=never --bench first >"$tmp_output" 2>&1)

    # Criterion output may still include ANSI sequences from downstream formatting.
    sed -E 's/\x1B\[[0-9;]*[[:alpha:]]//g' "$tmp_output" > "$tmp_clean"

    awk -v name="$benchmark_name" '
        $0 ~ ("^# " name "[[:space:]]*$") { p=1; print; next }
        p && /^A draft AI/ { exit }
        p { print }
    ' "$tmp_clean" > "$tmp_extract"

    if [[ -s "$tmp_extract" ]]; then
        cat "$tmp_extract" >> "$RESULTS_FILE"
        echo >> "$RESULTS_FILE"
    else
        printf 'failed to extract summary table for %s\n' "$benchmark_name" >&2
    fi

    rm -f "$tmp_output" "$tmp_clean" "$tmp_extract"
done
