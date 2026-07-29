#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/het/results

cargo bench --bench het-simple
cp target/criterion/het_simple/summary_het_simple.csv benches/het/results/simple.csv

cargo bench --bench het-advanced
cp target/criterion/het_advanced/summary_het_advanced.csv benches/het/results/advanced.csv
