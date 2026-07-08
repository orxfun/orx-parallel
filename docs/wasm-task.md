# Current State

Currently we can use orx-parallel in two ways:
* via rayon-core and wasm-bindgen-rayon. This is demonstrated by examples/wasm_demo_tsp.
* via pool/wasm_web2.rs. This is demonstrated by examples/wasm_demo_tsp2

As far as I could observe the prior is faster than the latter. The goal is to make them have similar performances.

## Step 1

To observe performance, we want to create examples/wasm_perf.rs example which will compare three variants:

1. rayon: rayon and wasm-bindgen-rayon (no orx-parallel)
2. rayon-orx: orx-parallel, rayon-core and wasm-bindgen-rayon, as in wasm_demo_tsp
3. orx: just orx-parallel, as in wasm_demo_tsp2.

We would like to solve a similar TSP problem with all of them and compare their WASM performances.

### Step 1 Implementation

Implemented at `examples/wasm_perf/` as a browser benchmark example with shared TSP logic:

1. rayon: `examples/wasm_perf/crate_rayon`
2. rayon-orx: `examples/wasm_perf/crate_rayon_orx`
3. orx: `examples/wasm_perf/crate_orx`

Shared algorithm/data source:

- `examples/wasm_perf/tsp_core` (deterministic locations + same 2-opt search implementation)

Browser harness:

- `examples/wasm_perf/web` (manual benchmark UI and structured output)

### Step 1 Benchmark Protocol

- Runtime target: Browser (Chrome/Firefox)
- Metrics: wall-clock computation time (ms) and throughput (iterations/s)
- Fairness rules:
	- same algorithm and data across variants
	- warmup runs excluded from measured statistics
	- fixed thread count per session
	- initialization excluded from timing
- Cases:
	- cities: 50, 75
	- iterations: 1000, 10000
	- warmups: 2
	- measured runs: 5
	- threads: run separate sessions for 4 and 8

### Step 1 Results Log

Thread count = 4

- pending

Thread count = 8

- pending

## Step 2

Improve performance of "orx" to make it comparable with options "rayon" and / or "rayon-orx:.
