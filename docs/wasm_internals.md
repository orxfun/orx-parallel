# WebAssembly Internals

This document describes the browser-threaded WebAssembly backend and how it
connects to the JavaScript host. For end-to-end usage, see the mini and TSP
applications in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).
They use the companion [`orx-parallel-wasm`](https://github.com/orxfun/orx-parallel-wasm)
package to build and package the bindings and to run computations through its
`ParallelWorker` client.

## Export matrix

The crate exposes different wasm items depending on feature flags and target configuration.

### `wasm` backend

When all of these hold:

- `feature = "wasm"`
- `target_arch = "wasm32"`

the crate exports:

- `WasmWebPool`

When `feature = "wasm-allocator"` is enabled for `wasm32`, it also exports:

- `WasmParallelAllocator`

Additionally, when `target_feature = "atomics"` is also enabled, it exports:

- `init_wasm_parallel_runtime(...)`
- `wasm_web_runtime_info()`
- `wasm_web_start_worker()`

The re-exports are wired through:

- `src/lib.rs`
- `src/pools/mod.rs`
- `src/pools/pool_impl/mod.rs`

## Default pool selection on wasm

`src/pools/global_pool.rs` selects the default global pool by feature set.

- On `wasm32` with `wasm`, the default pool type is `&'static WasmWebPool`.

The default runner uses this pool, so ordinary parallel iterator calls need no
explicit pool construction. The wasm runtime must still be initialized before
the first parallel computation.

## Backend implementation

The main implementation lives in `src/pools/pool_impl/wasm_web.rs`.

This backend is a custom worker-backed runtime.

### Core globals

The backend keeps process-wide wasm runtime state in three globals:

- `WASM_WEB3_THREAD_POOL_STATE`: whether initialization has begun
- `WASM_WEB3_THREAD_POOL_NUM_THREADS`: configured thread count
- `WASM_WEB3_RUNTIME`: shared runtime state stored in a `OnceLock<Arc<Inner>>`

`Inner` owns:

- shared worker state
- the number of spawned workers

The worker-shared state contains:

- a task queue
- an active scope pointer
- a shutdown flag
- a condition variable for waking workers

## Initialization flow in the main backend

`init_wasm_parallel_runtime(num_threads)` is the public wasm-bindgen entrypoint.
It delegates to `init_wasm_thread_pool(num_threads)` in the backend.

Its behavior is:

1. Normalize the thread count (`0` uses the crate's resource- and environment-based limit; a positive value is used as the requested count).
2. Record the count and initialize the shared runtime state.
3. Call into JavaScript to start workers; the returned `Promise` settles after they report ready or a startup error occurs.

Reinitialization policy:

- Same thread count: resolves immediately.
- Different thread count: rejects the returned `Promise`.

The JavaScript bridge is imported with:

```rust
#[wasm_bindgen(module = "/src/pools/pool_impl/wasm_web_start_workers.js")]
```

That module starts module workers and waits for each worker to report readiness.

## Worker bootstrap path

The JS bootstrap file is `src/pools/pool_impl/wasm_web_start_workers.js`.

Its job is to:

1. create module workers
2. send each worker the WASM initialization data and shared memory handle
3. wait for a ready/error/timeout result from each worker

Inside the worker helper:

- the generated wasm package is imported dynamically
- the generated package's default initializer is awaited with the shared memory supplied by the parent runtime
- the exported Rust worker entrypoint `wasm_web_start_worker()` is called

`orx-parallel-wasm` prepares the generated package for bundling. It copies this
helper beside the generated worker entry and adjusts the wasm-bindgen
initializer call so each pool worker uses the shared `WebAssembly.Memory`.
Bundler adapters then include the helper and generated bindings/WASM in the
output asset graph.

In the demos, `ParallelWorker` creates a dedicated application worker. That
worker loads the generated bindings, runs their default initializer, and calls
`init_wasm_parallel_runtime` before accepting calls to the configured method
allowlist. The Rust initialization then starts the pool workers described
above. Calls through one `ParallelWorker` are queued and run one at a time, so
the application can keep CPU-heavy work off its UI thread while Rust
parallelizes each computation internally.

The integration package interprets `threads: 0` as
`Math.max(1, navigator.hardwareConcurrency ?? 1)` before initializing Rust.
Code that calls `init_wasm_parallel_runtime(0)` directly instead uses the
crate's resource- and environment-based limit.

That exported Rust function enters the Rust-side `worker_loop(...)` and begins consuming queued tasks.

## Scoped execution model

The `wasm` backend implements `ThreadPool` for `WasmWebPool`.

Each parallel computation is wrapped in a scoped execution.

### Scope runtime

For each scoped computation, the backend creates a `ScopeRuntime` containing:

- `pending`: number of queued/running tasks
- completion synchronization primitives
- a panic slot

The scoped flow is:

1. create a new `ScopeRuntime`
2. publish its address as the active scope
3. run the user computation that schedules work
4. wait until pending task count returns to zero
5. clear the active scope
6. resume any panic captured either in user code or worker code

On `wasm32`, the calling thread waits for the atomic pending count to reach zero
with a spin loop. This keeps the iterator API synchronous from Rust's point of
view while work runs on browser workers.

### Task scheduling

`run(...)` does the following:

- increments the scope's pending count
- if the runtime is `inline_only`, runs the work immediately
- otherwise boxes the closure as a `Task`, pushes it into the queue, and notifies one worker

Workers repeatedly:

- wait for queued work
- pop one task
- execute it with `catch_unwind`
- record the first panic if one occurs
- decrement the pending count

### Why `inline_only` exists

The scope reference carries an `inline_only` flag, derived from whether any
workers were spawned.

If no workers are available, the backend can still execute the scoped tasks inline. This gives the pool a defined fallback mode instead of requiring a separate execution path at the iterator layer.

## Why initialization is explicit

On native targets, lazy pool creation is often acceptable.

In browser wasm, the runtime depends on external conditions that are not owned by Rust code alone:

- atomics-enabled wasm output
- shared memory support
- JS worker creation
- cross-origin isolation headers

The demo integration awaits initialization through `ParallelWorker.ready()`
before exposing computations. Applications that call the generated bindings
directly must call and await `init_wasm_parallel_runtime(...)` themselves.

## JavaScript packaging layer

The generated `wasm-bindgen` package is not, by itself, a complete application
integration. It contains the bindings glue, the WASM binary, and snippets that
spawn workers, but a browser build still needs to preserve those relationships
in its output asset graph.

`orx-parallel-wasm` provides bundler-neutral `buildWasm` and `prepareWasm`
functions, plus integrations for Vite, Webpack, Rspack, and Rollup. The build
command runs `wasm-pack` with the threaded-wasm flags; preparation adjusts and
copies the worker helper and records the generated assets in a manifest. The
adapters handle including those assets in the bundler output and configure the
development/hosting headers needed for cross-origin isolation.

The manual vanilla demo uses the package's build/preparation APIs and a small
server that sets the required headers. The other mini apps use the bundler
adapters.

## Relationship to the examples

The example apps keep browser concerns outside the computation crate.

That mirrors the runtime design:

- `orx-parallel` owns scheduling, the wasm pool, and scoped execution.
- `wasm_bindings` exposes computation methods; the wasm runtime initializer is available through the generated bindings.
- `orx-parallel-wasm` owns generated-package preparation, bundler integration, and the `ParallelWorker` client.
- The browser host owns the client lifecycle and deployment configuration.

This separation is not accidental; it matches the actual responsibility boundaries in the implementation.

## Practical implications for maintainers

When adjusting wasm support, the places that usually need to stay aligned are:

- Rust exports in `src/lib.rs`, `src/pools/mod.rs`, and `src/pools/pool_impl/mod.rs`.
- Pool selection in `src/pools/global_pool.rs`.
- Backend implementation in `src/pools/pool_impl/wasm_web.rs`.
- JS bootstrap in `src/pools/pool_impl/wasm_web_start_workers.js`.
- `orx-parallel-wasm` preparation and bundler adapters that package worker helper files.
- Host server configuration for COOP/COEP headers.

Most documentation drift happens when one of those layers changes without updating the others. The current mini and TSP examples in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos), together with the `orx-parallel-wasm` package README, are the best source of truth for a working browser-hosted setup.
