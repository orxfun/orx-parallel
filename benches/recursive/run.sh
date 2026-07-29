#!/usr/bin/env bash
set -euo pipefail

mkdir -p benches/recursive/results

cargo bench --bench recursive-file-system
cp target/criterion/recursive_tree_traversal/summary_recursive_file_system.csv benches/recursive/results/file_system.csv

cargo bench --bench recursive-tree-traversal
cp target/criterion/recursive_tree_traversal/summary_recursive_tree_traversal.csv benches/recursive/results/tree_traversal.csv
