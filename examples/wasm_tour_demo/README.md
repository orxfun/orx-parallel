# orx-parallel wasm tour demo

This demo shows how to run a parallel random tour search in the browser using `orx-parallel` wasm web threads.

## Structure

- `crate/`: Rust wasm compute module exporting `init_thread_pool`, `locations`, and `run_best_tour_par`.
- `web/`: Vite frontend that initializes wasm threads, runs the search, and renders the best tour.

## Build the wasm module

From repository root:

```bash
cd examples/wasm_tour_demo

cd web
npm run build:wasm
```

## Run frontend

```bash
cd web
npm install
npm run dev:full
```

Open the printed local URL. The Vite server is configured with COOP/COEP headers required for wasm threads.

## Notes

- `init_thread_pool(...)` must be awaited before running parallel workloads.
- The thread count is effectively fixed after first initialization for the process lifetime.
