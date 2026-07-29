#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench throughput-linear-log-collect
cargo bench --bench throughput-linear-log-reduce
