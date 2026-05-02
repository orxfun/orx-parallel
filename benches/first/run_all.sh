#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_TOML="$(cd "$SCRIPT_DIR/../.." && pwd)/Cargo.toml"

for benchmark_file in "$SCRIPT_DIR"/*.rs; do
    [ -e "$benchmark_file" ] || continue

    benchmark_basename="$(basename "$benchmark_file")"

    benchmark_name="$({ sed -n 's/.*Exp\.bench(c, "\([^"]*\)",.*/\1/p' "$benchmark_file"; } | head -n 1)"

    if [[ -z "$benchmark_name" ]]; then
        printf 'could not parse benchmark name from %s\n' "$benchmark_file" >&2
        continue
    fi

    sed -i '/^\[\[bench\]\]$/,/^\[/ s|^path = "benches/first/.*\.rs"$|path = "benches/first/'"$benchmark_basename"'"|' "$CARGO_TOML"

    printf '%s -> %s\n' "$benchmark_basename" "$benchmark_name"
done
