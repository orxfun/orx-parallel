#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench recursive-file-system
cargo bench --bench recursive-tree-traversal
