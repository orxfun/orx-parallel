#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench fallible-validation-result
cargo bench --bench fallible-validation-option
