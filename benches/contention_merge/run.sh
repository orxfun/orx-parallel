#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/contention_merge/results

cargo bench --bench contention-merge-grouped-topk
cp target/criterion/contention_merge_grouped_topk/summary_contention_merge_grouped_topk.csv benches/contention_merge/results/grouped_topk.csv
