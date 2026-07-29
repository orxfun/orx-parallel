#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/stateful_using/results

cargo bench --bench stateful-using-monte-carlo-batch
cp target/criterion/stateful_using_monte_carlo_batch/summary_stateful_using_monte_carlo_batch.csv benches/stateful_using/results/monte_carlo_batch.csv
