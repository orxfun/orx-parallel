#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench early-exit-suspicious-first
cargo bench --bench early-exit-suspicious-find-any
