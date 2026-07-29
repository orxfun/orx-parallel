#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/reduce/results

cargo bench --bench reduce-f
cp target/criterion/reduce_f/summary_reduce_f.csv benches/reduce/results/f.csv

cargo bench --bench reduce-id
cp target/criterion/reduce_id/summary_reduce_id.csv benches/reduce/results/id.csv

cargo bench --bench reduce-l
cp target/criterion/reduce_l/summary_reduce_l.csv benches/reduce/results/l.csv

cargo bench --bench reduce-m
cp target/criterion/reduce_m/summary_reduce_m.csv benches/reduce/results/m.csv

cargo bench --bench reduce-mf
cp target/criterion/reduce_mf/summary_reduce_mf.csv benches/reduce/results/mf.csv

cargo bench --bench reduce-mfm
cp target/criterion/reduce_mfm/summary_reduce_mfm.csv benches/reduce/results/mfm.csv

cargo bench --bench reduce-mfmf
cp target/criterion/reduce_mfmf/summary_reduce_mfmf.csv benches/reduce/results/mfmf.csv
