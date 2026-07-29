#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench contention-merge-grouped-topk
