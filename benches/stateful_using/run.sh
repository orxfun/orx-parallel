#!/usr/bin/env bash
set -euo pipefail

cargo bench --bench stateful-using-monte-carlo-batch
