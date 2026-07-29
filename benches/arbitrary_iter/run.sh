#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/arbitrary_iter/results

cargo bench --bench arbitrary-iter-map-set-processing
cp target/criterion/arbitrary_iter_map_set_processing/summary_arbitrary_iter_map_set_processing.csv benches/arbitrary_iter/results/map_set_processing.csv
