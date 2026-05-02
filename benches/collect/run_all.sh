#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_TOML="$(cd "$SCRIPT_DIR/../.." && pwd)/Cargo.toml"
RESULTS_FILE="$SCRIPT_DIR/run_results.md"

: > "$RESULTS_FILE"

update_bench_path() {
    local bench_name="$1"
    local bench_path="$2"
    local tmp_toml

    tmp_toml=$(mktemp)

    awk -v bench_name="$bench_name" -v bench_path="$bench_path" '
        /^\[\[bench\]\]$/ {
            in_bench=1
            matched_name=0
            print
            next
        }

        in_bench && /^\[[^[]/ {
            in_bench=0
            matched_name=0
        }

        in_bench && /^name = / {
            matched_name = ($0 == "name = \"" bench_name "\"")
        }

        in_bench && matched_name && /^path = / {
            print "path = \"" bench_path "\""
            updated=1
            next
        }

        { print }

        END {
            exit updated ? 0 : 1
        }
    ' "$CARGO_TOML" > "$tmp_toml"

    mv "$tmp_toml" "$CARGO_TOML"
}

if grep -q '^name = "collect"$' "$CARGO_TOML"; then
    BENCH_TARGET="collect"
else
    BENCH_TARGET="first"
fi

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

    update_bench_path "$BENCH_TARGET" "benches/collect/$benchmark_basename"

    printf '[%d / %d] %s\n' "$benchmark_index" "$total_benchmarks" "$benchmark_name"

    tmp_output=$(mktemp)
    tmp_clean=$(mktemp)
    tmp_extract=$(mktemp)
    (cd "$SCRIPT_DIR/../.." && cargo bench --color=never --bench "$BENCH_TARGET" >"$tmp_output" 2>&1)

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
