# orx-parallel wasm TSP vanilla

This example shows the recommended web structure for `orx-parallel` with a Vite + TypeScript frontend:

- `computation/` contains pure Rust TSP logic
- `wasm_bindings/` exposes a thin wasm API for that computation
- `ui/` is the browser UI that consumes the wasm bindings

The same structure works for other parallelizable Rust workloads too. The important part is the separation: keep the algorithm in Rust, keep the wasm layer thin, and keep the UI focused on orchestration.

## Project responsibilities

### `computation/`

This crate contains the actual TSP implementation. It is the best place to add benchmarks, unit tests, and algorithm changes.

Use this crate for anything that should stay independent from the web platform:

- instance generation
- tour construction and improvement
- sequential and parallel search strategies

### `wasm_bindings/`

This crate is the boundary between Rust and JavaScript. It should stay thin.

Its job is to:

- expose wasm-safe functions such as `locations`, `init_parallel_runtime`, and `run_search`
- serialize and deserialize values at the edge
- initialize the parallel runtime before the first parallel search

### `ui/`

This is the browser application. It owns the page, worker lifecycle, controls, and rendering.

The UI should call into the wasm package, but it should not reimplement TSP logic or touch the computation crate directly.

## Execution flow

1. The UI creates or loads a TSP instance.
2. The UI sends the request to a worker.
3. The worker calls `init()` for the generated wasm package.
4. If the request is parallel, the worker calls `init_parallel_runtime(thread_count)` before the first `run_search`.
5. `run_search` executes the Rust computation and returns the result to the worker.
6. The worker posts the result back to the UI.

The worker can be short-lived, or it can be kept alive and reused across searches. In either case, each worker that runs parallel work must call `init_parallel_runtime` once before its first parallel search.

## Important rules for parallel wasm on the web

Parallel wasm in the browser has a few hard requirements. Missing any one of them usually leads to code that still runs, but no longer uses parallel execution.

### 1. Build wasm with thread support enabled

The wasm build must target shared-memory/threaded wasm. In practice, that means using the build setup from `ui/package.json` and the Rust configuration in `computation/` and `wasm_bindings/`.

If you change the Rust code, rebuild the wasm package before running the UI again.

### 2. Serve the app with cross-origin isolation headers

Browser threads require `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers. Without them, `SharedArrayBuffer` is unavailable and the threaded path cannot work.

A plain static server is usually not enough. Use the Vite dev server or a server that sends the same headers for the built app.

```ts
// ui/vite.config.ts
import { defineConfig } from "vite";

export default defineConfig({
    server: {
        headers: {
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp"
        }
    }
});
```

### 3. Initialize the parallel runtime once before the first parallel search

Call `init_parallel_runtime` once in each worker that will execute parallel work.

Do not assume that one worker initializing the runtime automatically prepares every other worker. If you create a new worker, that worker must initialize its own runtime before parallel search.

## Testing strategy

The separation also makes testing straightforward:

- test the algorithm in `computation/` with ordinary Rust tests
- test the wasm boundary in `wasm_bindings/`
- test UI behavior in `ui/`

That division is deliberate. It lets you validate the core search independently from the browser and only use wasm tests where they are actually needed.

## Read next

- [computation README](./computation/README.md)
- [wasm bindings README](./wasm_bindings/README.md)
- [UI README](./ui/README.md)

The `ui/README.md` contains the exact setup and run commands for the browser app, while `wasm_bindings/README.md` documents the wasm API surface.
