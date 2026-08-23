# components

This crate contains the Yew UI for the wasm TSP example.

It is responsible for rendering the page, managing UI state, and invoking the search through the browser worker bridge.

## Responsibilities

- render the interactive UI with Yew
- keep search settings and view state in Rust
- call into the generated wasm exports such as `start_app`, `init_wasm_parallel_runtime`, and `run_search`
- hand off worker lifecycle concerns to the JavaScript host application

## How it fits into the example

This crate sits between the browser app and the computation bindings:

- `app/src/main.ts` loads the generated wasm package and calls `start_app()`
- `components/` renders the UI and prepares search requests
- `app/src/search-runner.ts` exposes a JavaScript function on `globalThis` so the Yew UI can trigger a worker-backed search
- `app/src/search-worker.ts` initializes wasm in a worker and calls `run_search`

That split is deliberate. It keeps UI state and presentation in Rust while leaving browser-specific worker setup and bundler concerns in the Vite app.

## Exported entry point

The main browser entry point exported by this crate is:

- `start_app()`: mounts the Yew application into the page after the generated wasm package has been initialized

This crate also calls into the wasm bindings to execute searches, but it does that internally as part of the UI flow rather than exposing a second public API layer.

## Parallel execution

The UI itself does not create the thread pool directly. Instead, it sends a request to the TypeScript worker bridge, and the worker calls `init_wasm_parallel_runtime(thread_count)` before the first parallel `run_search`.

This separation matters because browser workers own their own wasm instances and runtime initialization.

## Build relationship

The browser app builds this crate with:

```bash
wasm-pack build ../components --target web --out-dir ../app/pkg
```

That command generates the wasm artifact plus `app/pkg/components.js`, which the browser host loads from `app/src/main.ts` and `app/src/search-worker.ts`.