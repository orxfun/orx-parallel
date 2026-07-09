# orx-parallel wasm perf

Step 1 benchmark harness for comparing wasm variants on the same TSP workload:

- rayon: pure rayon + wasm-bindgen-rayon
- rayon-orx: orx-parallel + rayon-core + wasm-bindgen-rayon backend
- orx: orx-parallel wasm-web-threads2 backend
- orx3: orx-parallel wasm-web-threads3 backend

All variants reuse the same TSP algorithm and deterministic city generation from `tsp_core`.

## Layout

- `tsp_core/`: shared TSP logic and location generation
- `crate_rayon/`: wasm adapter for rayon-only variant
- `crate_rayon_orx/`: wasm adapter for rayon-orx variant
- `crate_orx/`: wasm adapter for orx variant
- `crate_orx3/`: wasm adapter for orx3 variant
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

4. Open the shown URL and run benchmark with these Step 1 defaults:

- cities: `50,75`
- iterations: `1000,10000`
- warmups: `2`
- measured runs: `5`
- seed: `42`

5. Run two separate sessions for fairness:

- session A: threads = `4`
- session B: threads = `8`

Thread pool init is one-time per module; reload the page before switching thread count.

## Output

The page prints:

- median and mean elapsed ms
- median and mean throughput (iterations/s)
- raw JSON samples for documentation

Copy the output into `docs/wasm-task.md` under Step 1 results.
