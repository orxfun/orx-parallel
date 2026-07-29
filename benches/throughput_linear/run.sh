#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/throughput_linear/results

cargo bench --bench throughput-linear-log-collect
cp target/criterion/throughput_linear_log_collect/summary_throughput_linear_log_collect.csv benches/throughput_linear/results/log_collect.csv

cargo bench --bench throughput-linear-log-reduce
cp target/criterion/throughput_linear_log_reduce/summary_throughput_linear_log_reduce.csv benches/throughput_linear/results/log_reduce.csv
