#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/early_exit/results

cargo bench --bench early-exit-suspicious-first
cp target/criterion/early_exit_suspicious_first/summary_early_exit_suspicious_first.csv benches/early_exit/results/suspicious_first.csv

cargo bench --bench early-exit-suspicious-find-any
cp target/criterion/early_exit_suspicious_find_any/summary_early_exit_suspicious_find_any.csv benches/early_exit/results/suspicious_find_any.csv
