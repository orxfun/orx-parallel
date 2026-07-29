#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/fallible/results

cargo bench --bench fallible-validation-result
cp target/criterion/fallible_validation_result/summary_fallible_validation_result.csv benches/fallible/results/validation_result.csv

cargo bench --bench fallible-validation-option
cp target/criterion/fallible_validation_option/summary_fallible_validation_option.csv benches/fallible/results/validation_option.csv
