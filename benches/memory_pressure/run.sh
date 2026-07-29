#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench memory-pressure-string-formatting
