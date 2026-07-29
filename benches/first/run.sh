#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/first/results

cargo bench --bench first-f
cp target/criterion/first_f/summary_first_f.csv benches/first/results/f.csv

cargo bench --bench first-ff
cp target/criterion/first_ff/summary_first_ff.csv benches/first/results/ff.csv

cargo bench --bench first-fff
cp target/criterion/first_fff/summary_first_fff.csv benches/first/results/fff.csv

cargo bench --bench first-i
cp target/criterion/first_i/summary_first_i.csv benches/first/results/i.csv

cargo bench --bench first-id
cp target/criterion/first_id/summary_first_id.csv benches/first/results/id.csv

cargo bench --bench first-lf
cp target/criterion/first_lf/summary_first_lf.csv benches/first/results/lf.csv

cargo bench --bench first-mf
cp target/criterion/first_mf/summary_first_mf.csv benches/first/results/mf.csv

cargo bench --bench first-mfmf
cp target/criterion/first_mfmf/summary_first_mfmf.csv benches/first/results/mfmf.csv

cargo bench --bench first-mi
cp target/criterion/first_mi/summary_first_mi.csv benches/first/results/mi.csv
