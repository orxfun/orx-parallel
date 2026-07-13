# orx-parallel wasm perf

Step 1 benchmark harness for comparing wasm variants on the same TSP workload:

- rayon: pure rayon + wasm-bindgen-rayon
- orx-rayon: orx-parallel wasm-web-threads-experimental backend
- orx: orx-parallel wasm-web-threads backend

All variants reuse the same TSP algorithm and deterministic city generation from `tsp_core`.

## Layout

- `tsp_core/`: shared TSP logic and location generation
- `crate_rayon/`: wasm adapter for rayon-only variant
- `crate_orx/`: wasm adapter for orx variant
- `crate_rayon_orx/`: wasm adapter for orx-rayon variant
- `web/`: browser benchmark runner (manual execution)

## Run

1. Install prerequisites:

```bash
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install wasm-pack
```

2. Install web dependencies:

```bash
cd examples/wasm_perf/web
npm install
```

3. Build all wasm packages and start the benchmark UI:

```bash
cd examples/wasm_perf/web
npm run dev:full
```

	Select benchmark variant from command line when starting the app:

```bash
cd examples/wasm_perf/web
PAR_POOL_VARIANT=orx npm run dev:full

# optional: fix thread count from environment (default: 4)
PAR_POOL_VARIANT=orx PAR_NUM_THREADS=8 npm run dev:full
```

Supported values: `rayon`, `orx-rayon`, `orx`.

`PAR_NUM_THREADS` supports values in `1..16`.

`npm run preview` also rebuilds first, so `PAR_POOL_VARIANT=orx PAR_NUM_THREADS=8 npm run preview` will serve the selected variant and thread count.

4. Open the shown URL and run benchmark with these Step 1 defaults:

- cities: `50,75`
- iterations: `1000,10000`
- warmups: `2`
- measured runs: `5`
- seed: `42`

5. Run two separate sessions for fairness:

- session A: `PAR_NUM_THREADS=4`
- session B: `PAR_NUM_THREADS=8`

Thread pool init starts automatically on page load and is one-time per module; reload the page after changing `PAR_POOL_VARIANT` or `PAR_NUM_THREADS`.

## Output

The page prints:

- median and mean elapsed ms
- median and mean throughput (iterations/s)
- raw JSON samples for documentation

Copy the output into `docs/wasm-task.md` under Step 1 results.
