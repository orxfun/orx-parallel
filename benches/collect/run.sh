#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/collect/results

cargo bench --bench collect-f
cp target/criterion/collect_f/summary_collect_f.csv benches/collect/results/f.csv

cargo bench --bench collect-id
cp target/criterion/collect_id/summary_collect_id.csv benches/collect/results/id.csv

cargo bench --bench collect-l
cp target/criterion/collect_l/summary_collect_l.csv benches/collect/results/l.csv

cargo bench --bench collect-m
cp target/criterion/collect_m/summary_collect_m.csv benches/collect/results/m.csv

cargo bench --bench collect-mf
cp target/criterion/collect_mf/summary_collect_mf.csv benches/collect/results/mf.csv

cargo bench --bench collect-mfm
cp target/criterion/collect_mfm/summary_collect_mfm.csv benches/collect/results/mfm.csv

cargo bench --bench collect-mfmf
cp target/criterion/collect_mfmf/summary_collect_mfmf.csv benches/collect/results/mfmf.csv
