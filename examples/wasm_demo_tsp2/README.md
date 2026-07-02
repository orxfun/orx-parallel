# orx-parallel wasm demo tsp2

This demo shows the `wasm-web-threads2` path for running `orx-parallel` in the browser with wasm threads.

It mirrors `examples/wasm_demo_tsp` in structure and behavior, but uses the new backend that does not depend on `rayon-core` or `wasm-bindgen-rayon`.

## What this demo contains

- `crate/`: Rust compute module compiled to wasm.
- `web/`: Vite + TypeScript UI that loads wasm, initializes the thread pool, and runs search jobs.

## How to run

Use the same workflow as the existing demo:

1. Install prerequisites.
2. Build the wasm package from `web/`.
3. Start the Vite dev server.
4. Open the printed URL and run parallel or sequential search.

The runtime startup flow is the same:

1. `await init()`
2. `await init_parallel_runtime(...)`
3. run parallel computations
