# wasm_bindings

This crate provides the WebAssembly bindings for the `computation` crate so the UI can solve TSP instances from JavaScript.

Its only responsibility is to expose a wasm-friendly API. All TSP logic lives in the `computation` crate, which keeps the implementation modular, easier to test, and easier to reuse outside the UI.

## Exposed API

- `locations(seed, num_cities)`: generates a random TSP instance and returns a JS array of locations.
- `init_parallel_runtime(num_threads)`: initializes the shared thread pool for parallel execution.
- `run_search(parallelize, iterations, seed, threads, chunk_size, locations)`: runs the search and returns the best tour summary.

The `locations` argument passed to `run_search` must be a JS array of objects shaped like `{ x: number, y: number }`.

The returned object contains:

- `best_tour`
- `best_distance`
- `iterations`
- `elapsed_ms`

## Parallel execution

Parallel search requires a wasm target with thread support enabled, including atomic operations and shared memory. In practice, this means building for `wasm32-unknown-unknown` with the appropriate threading support in the browser or host environment.

If you intend to use parallel search, call `init_parallel_runtime` once before the first `run_search` call in each worker that will run parallel work.

If the target does not support atomics and shared memory, `init_parallel_runtime` cannot succeed.

## Testing

This crate can be tested with:

```bash
cargo test -p wasm_bindings --target wasm32-unknown-unknown
```
