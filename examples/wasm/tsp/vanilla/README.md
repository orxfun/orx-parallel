# orx-parallel wasm TSP vanilla

This example is the recommended shape for using `orx-parallel` in the web with a JS-framework:

- `computation/` contains pure Rust logic
  - knows nothing about JavaScript, wasm-bindgen, or the DOM
- `wasm_bindings/` exposes a thin wasm API for that computation
  - only translates between JS values and Rust types, and initializes the runtime needed for parallel search
- `ui/` is the TypeScript/Vite frontend that consumes the wasm bindings
  - only drives the user experience, starts workers, and calls the generated wasm bindings

In this example, `computation` crate contains functions to solve the traveling salesperson (TSP) problem. However, it can be any crate that contains the parallelizable computation.

The split is intentional. It keeps the computation testable without the browser, keeps the wasm layer small, and keeps the UI focused on presentation and orchestration.

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

## Important rules for parallel wasm on the web

Parallel wasm in the browser has a few hard requirements. Missing any one of them usually leads to code that still runs, but no longer uses parallel execution.

### 1. Build wasm with thread support enabled

The wasm build must target shared-memory/threaded wasm. In practice, that means using the build setup from `ui/package.json` and the Rust configurations of `computation/` and `wasm_bindings/`.

If you change the Rust code, rebuild the wasm package before running the UI again.


```toml
# computation/Cargo.toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false }

[features]
default = []
wasm-web-threads = ["orx-parallel/wasm-web-threads"]
```

```toml
# wasm_bindings/Cargo.toml
[dependencies]
computation = { path = "../computation", features = ["wasm-web-threads"] }
```

```json
// ui/package.json
{
    "scripts": {
        "build:wasm": "RUSTUP_TOOLCHAIN=nightly RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' wasm-pack build ../wasm_bindings --target web --out-dir ../ui/pkg -- -Z build-std=panic_abort,std",
    }
}
```

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

```ts
// ui/src/search-worker.ts
import init, { init_parallel_runtime, run_search } from "../pkg/wasm_bindings.js";
import type { SearchRequest, SearchResponse } from "./shared-types.js";

self.addEventListener("message", async (event: MessageEvent<SearchRequest>) => {
    const settings = event.data.settings;
    await init();

    if (settings.mode === "parallel") {
        await init_parallel_runtime(settings.threads);
    }

    let parallelize = settings.mode === "parallel";
    const result = run_search(/*...*/);
});
```

## Testing strategy

The separation also makes testing straightforward:

- test the algorithm in `computation/` with ordinary Rust tests
- test the wasm boundary in `wasm_bindings/`
- test UI behavior in `ui/`

That division is deliberate. It lets you validate the core search independently from the browser and only use wasm tests where they are actually needed.

## Read next

Please also check the brief notes in readme files of three of these components.

- [computation README](./computation/README.md)
- [wasm bindings README](./wasm_bindings/README.md)
- [UI README](./ui/README.md)

The `ui/README.md` contains the exact setup and run commands for the browser app, while `wasm_bindings/README.md` documents the wasm API surface.
