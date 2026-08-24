# WebAssembly Internals

This document explains how wasm support in `orx-parallel` is structured.

It focuses on the Rust runtime and its JavaScript boundary. The companion
[`orx-parallel-wasm`](https://github.com/orxfun/orx-parallel-wasm) package owns
the browser packaging layer: it runs or prepares `wasm-pack` output, adapts
worker assets for the selected bundler, and provides the `ParallelWorker`
client used by the examples.

## Export matrix

The crate exposes different wasm items depending on feature flags and target configuration.

### `wasm` backend

When all of these hold:

- `feature = "wasm"`
- `target_arch = "wasm32"`

the crate exports:

- `WasmWebPool`

Additionally, when `target_feature = "atomics"` is also enabled, it exports:

- `init_wasm_thread_pool(...)`
- `wasm_web_runtime_info()`
- `wasm_web_start_worker()`

The re-exports are wired through:

- `src/lib.rs`
- `src/pool/mod.rs`
- `src/pool/pool_impl/mod.rs`

## Default pool selection on wasm

`src/pool/global_pool.rs` selects the default global pool by feature set.

- on `wasm32` with `wasm`, the default pool type is `&'static WasmWebPool`

This means ordinary parallel iterator calls can use the wasm backend without the application explicitly constructing a pool value, as long as runtime initialization has already happened.

## Backend implementation

The main implementation lives in `src/pool/pool_impl/wasm_web.rs`.

This backend is a custom worker-backed runtime.

### Core globals

The backend keeps process-wide wasm runtime state in three globals:

- `WASM_WEB3_THREAD_POOL_STATE`: whether initialization has happened
- `WASM_WEB3_THREAD_POOL_NUM_THREADS`: configured thread count
- `WASM_WEB3_RUNTIME`: the shared runtime state stored in a `OnceLock<Arc<Inner>>`

`Inner` owns:

- shared worker state
- the number of spawned workers

The worker-shared state contains:

- a task queue
- an active scope pointer
- a shutdown flag
- a condition variable for waking workers

## Initialization flow in the main backend

`init_wasm_thread_pool(num_threads)` is the explicit entrypoint.

Its behavior is:

1. normalize thread count
2. mark the wasm runtime initialized
3. create the shared runtime state
4. call into JavaScript to start workers

Thread count normalization:

- `num_threads = 0` means auto
- auto uses `crate::pool::env::max_num_threads_by_env_and_resource()`

Reinitialization policy:

- same thread count: resolves immediately
- different thread count: rejects the returned `Promise`

The JavaScript bridge is imported with:

```rust
#[wasm_bindgen(module = "/src/pool/pool_impl/wasm_web_start_workers.js")]
```

That JS module is responsible for spawning module workers and waiting until each worker reports readiness.

## Worker bootstrap path

The JS bootstrap file is `src/pool/pool_impl/wasm_web_start_workers.js`.

Its job is to:

1. create module workers
2. send each worker the WASM initialization data and shared memory handle
3. wait for a ready/error/timeout result from each worker

Inside the worker helper:

- the generated wasm package is imported dynamically
- the generated package's default initializer is awaited with the shared memory
	supplied by the parent runtime
- the exported Rust worker entrypoint `wasm_web_start_worker()` is called

`orx-parallel-wasm` prepares copies of this helper for bundler output. During
preparation it replaces the package-directory placeholder with the actual
generated bindings entry, copies the helper beside the generated worker entry,
and keeps the worker's package import and WASM assets in the same emitted asset
graph. The worker therefore initializes its own generated JS/WASM module while
sharing the `WebAssembly.Memory` created by the parent runtime.

The package's `ParallelWorker` client creates the top-level module worker,
sends it the bindings URL and requested thread count, and serializes calls made
through that client. The generated nested helpers then create the workers owned
by the Rust pool. This gives the application one client-facing worker boundary
while the Rust runtime manages its internal worker pool.

That exported Rust function enters the Rust-side `worker_loop(...)` and begins consuming queued tasks.

## Scoped execution model

The `wasm` backend implements `ParThreadPool` for `WasmWebPool`.

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

This is the key mechanism that keeps the external iterator API synchronous from Rust's point of view even though the work is running across browser workers.

### Task scheduling

`run_in_scope(...)` does the following:

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

The scope reference carries an `inline_only` flag, derived from whether any workers were spawned.

If no workers are available, the backend can still execute the scoped tasks inline. This gives the pool a defined fallback mode instead of requiring a separate execution path at the iterator layer.

## Why initialization is explicit

On native targets, lazy pool creation is often acceptable.

In browser wasm, the runtime depends on external conditions that are not owned by Rust code alone:

- atomics-enabled wasm output
- shared memory support
- JS worker creation
- cross-origin isolation headers

Surfacing initialization directly through `init_wasm_thread_pool(...)` makes these preconditions explicit and moves failures closer to application startup.

## JavaScript packaging layer

The generated `wasm-bindgen` package is not, by itself, a complete application
integration. It contains the bindings glue, the WASM binary, and snippets that
spawn workers, but a browser build still needs to preserve those relationships
in its output asset graph.

`orx-parallel-wasm` provides two levels of support:

- `buildWasm` and `prepareWasm` are bundler-neutral APIs. They build or prepare
	a generated package and write its asset manifest.
- The Vite, Webpack, Rspack, and Rollup adapters emit those assets, rewrite
	worker imports for their output layouts, create stable entries without
	colliding with the generated package entry, and provide COOP/COEP headers.

An application can use the neutral APIs directly. The
`examples/wasm/mini/vanilla-manual` example does this in `build.mjs` and uses
`server.mjs` to serve the output with the required headers. The other mini
examples use the bundler adapters.

## Relationship to the examples

The example apps keep browser concerns outside the computation crate.

That mirrors the runtime design:

- `orx-parallel` owns scheduling and scoped execution
- `wasm_bindings` exposes a small API such as `init_wasm_parallel_runtime(...)`
- `orx-parallel-wasm` owns generated-package preparation, bundler integration,
  and the `ParallelWorker` client
- the browser host owns the client lifecycle and deployment configuration

This separation is not accidental; it matches the actual responsibility boundaries in the implementation.

## Practical implications for maintainers

When adjusting wasm support, the places that usually need to stay aligned are:

- Rust exports in `src/lib.rs` and `src/pool/mod.rs`
- backend implementation in `src/pool/pool_impl/wasm_web.rs`
- JS bootstrap in `src/pool/pool_impl/wasm_web_start_workers.js`
- `orx-parallel-wasm` preparation and bundler adapters that package worker helper files
- host server configuration for COOP/COEP headers

Most documentation drift happens when one of those layers changes without updating the others. The current mini and TSP examples, together with the `orx-parallel-wasm` package README, are the best source of truth for a working browser-hosted setup.