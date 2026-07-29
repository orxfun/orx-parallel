#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/memory_pressure/results

cargo bench --bench memory-pressure-string-formatting
cp target/criterion/memory_pressure_string_formatting/summary_memory_pressure_string_formatting.csv benches/memory_pressure/results/string_formatting.csv
