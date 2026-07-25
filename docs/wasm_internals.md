# WebAssembly Internals

This document focuses on how `orx-parallel` supports parallel computation in browser-hosted wasm builds.

If you only want to enable and use wasm support, see [wasm.md](wasm.md).

## Public surface

When the corresponding feature is enabled on `wasm32`, the crate exposes:

- `WasmWebPool` for `wasm`
- `init_thread_pool(...)` for explicit runtime initialization on atomics-enabled builds

The exports are wired in `src/lib.rs`, `src/pool/mod.rs`, and `src/pool/pool_impl/mod.rs`.

## Main backend: `wasm`

The main backend lives in `src/pool/pool_impl/wasm_web.rs`.

Its design is specific to browser workers:

- a single runtime is stored in a `OnceLock`
- worker state is shared through Rust synchronization primitives
- tasks are queued into a scoped runtime
- worker startup is delegated to JavaScript via `wasm_bindgen`

At initialization time, `init_thread_pool(...)`:

1. decides the worker count,
2. creates the shared runtime state,
3. asks JavaScript to spawn module workers,
4. records the initialized runtime for later scoped computations.

The JavaScript bridge is imported from `src/pool/pool_impl/wasm_web_start_workers.js`.

### Worker lifecycle

The main backend exports a worker entrypoint so the generated wasm package can bootstrap newly created workers.

Once workers are ready, scoped jobs are pushed into a queue and consumed by worker loops. Parallel work is completed within a scope boundary so the caller does not continue until all submitted tasks finish.

## Why initialization is explicit

Native thread pools are often created lazily on first use. Browser wasm is less forgiving.

The crate requires explicit initialization because browser-thread support depends on:

- atomics-enabled wasm output,
- shared memory,
- worker creation from JavaScript,
- and browser isolation headers that allow `SharedArrayBuffer`.

Failing late inside an ordinary parallel call would make setup issues harder to diagnose, so the initialization step is surfaced directly.

## Runtime assumptions and failure modes

Both wasm backends assume:

- `target_arch = "wasm32"`
- atomics-enabled builds for actual browser threading
- a browser environment that supports worker-based shared-memory wasm

Common failure modes are:

- building wasm without atomics,
- forgetting to await `init_thread_pool(...)`,
- serving the app without cross-origin isolation headers,
The crate guards some of these with feature gating and explicit panics; the rest are environmental constraints that the host application must satisfy.

## Relationship to the examples

The framework examples under [examples/wasm/tsp](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp) intentionally keep the browser-specific orchestration outside the computation crate.

That separation mirrors the backend design:

- `orx-parallel` owns parallel execution,
- the wasm bindings expose a small initialization and execution API,
- the browser host owns worker lifecycle and HTTP headers.

This keeps the algorithm code portable while leaving browser setup at the application boundary.