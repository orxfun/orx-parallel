# orx-parallel wasm tsp vanilla

This example shows the TSP demo from `examples/arch/wasm_demo_tsp` in a plain Vite + TypeScript + DOM/CSS frontend.

## What it shows

- Rust wasm compute module
- Browser thread-pool initialization on startup
- Parallel and sequential TSP runs
- `num_threads` and `chunk_size` controls in the UI
- `wasm-web-threads` backend, not the experimental backend

## Layout

- `crate/`: Rust compute module compiled to wasm
- `web/`: Vite + TypeScript UI with vanilla DOM/CSS

## Run

1. Install prerequisites

```bash
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install wasm-pack
```

2. Install frontend dependencies

```bash
cd examples/wasm/tsp/vanilla/web
npm install
```

3. Start the app

```bash
cd examples/wasm/tsp/vanilla/web
ORX_PARALLEL_MAX_NUM_THREADS=16 npm run dev:full
```

Open the printed URL and choose sequential or parallel search.

## Startup threads

The frontend reads `ORX_PARALLEL_MAX_NUM_THREADS` through Vite and uses it to initialize the wasm thread pool once on page load.

If the variable is not set, the app falls back to 16.
