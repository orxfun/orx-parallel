#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for benchmark_file in "$SCRIPT_DIR"/*.rs; do
    [ -e "$benchmark_file" ] || continue

    benchmark_name="$({ sed -n 's/.*Exp\.bench(c, "\([^"]*\)",.*/\1/p' "$benchmark_file"; } | head -n 1)"

    if [[ -z "$benchmark_name" ]]; then
        printf 'could not parse benchmark name from %s\n' "$benchmark_file" >&2
        continue
    fi

    printf '%s -> %s\n' "$(basename "$benchmark_file")" "$benchmark_name"
done
