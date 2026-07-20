# Vite + TypeScript UI for `orx-parallel`

This demo uses **Vite** with **TypeScript** to drive a wasm-backed TSP search UI.

It illustrates enabling parallel computation using `orx-parallel` with wasm.

## wasm bindings

The wasm package is built into `./pkg` with the `build:wasm` script:

- `wasm-pack build ../wasm_bindings --target web --out-dir ../ui/pkg`
- the Rust build uses nightly and enables shared-memory/threaded wasm flags

The UI imports the generated bindings from `./pkg/wasm_bindings.js`:

- call `init()` once before using wasm exports
- use `locations(seed, num_cities)` to generate the TSP points
- use `run_search(...)` for the actual computation

## search worker

`src/search-worker.ts` is a module worker created from `src/main.ts` with `new Worker(new URL("./search-worker.ts", import.meta.url), { type: "module" })`.

Inside the worker:

- import `init`, `init_parallel_runtime`, and `run_search` from `./pkg/wasm_bindings.js`
- call `init()` in the worker before touching wasm exports
- call `init_parallel_runtime(threadCount)` before the first parallel search in that worker module
- send search results back to the main thread with `postMessage`

In this example, the thread pool is created per search-worker module. That means each worker owns its own pool, and a new worker implies a new pool.

Alternatively, one can create a persistent search worker with the thread pool created only once.

## config files that matter

- `package.json` defines the useful scripts:
  - `dev:full` builds wasm first, then starts Vite
  - `dev` starts Vite only
  - `build` runs TypeScript typechecking before the production build
- `vite.config.ts` uses `worker.format: "es"` so the module worker is emitted in ESM form
- `vite.config.ts` also sets `server.headers` for `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy`; this is required for cross-origin isolation, which in turn is required for shared-memory wasm and browser threads
- `tsconfig.json` uses `moduleResolution: "Bundler"` and `types: ["vite/client"]` so the generated wasm package and Vite imports typecheck cleanly

## minimal flow

1. Build the wasm bindings into `./pkg`.
2. Load the UI through Vite.
3. Initialize wasm in the main thread and in the worker.
4. Send the point set to the worker.
5. Run sequential or parallel search inside the worker.