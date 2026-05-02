#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for benchmark_file in "$SCRIPT_DIR"/*.rs; do
    [ -e "$benchmark_file" ] || continue
    :
done
