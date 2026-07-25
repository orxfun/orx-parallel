# Using orx-parallel in WebAssembly

This guide focuses on using `orx-parallel` in browser-hosted wasm builds.

Published demo: [orx-parallel-wasm-demo-tsp.pages.dev](https://orx-parallel-wasm-demo-tsp.pages.dev/).

If you want to understand how the wasm backends work internally, see [wasm_internals.md](wasm_internals.md).

## Which feature to enable

For browser threads, enable the `wasm` feature:

```toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false, features = ["wasm"] }
```

That is the recommended feature for web wasm support in this crate.

## When the feature should be optional

If a crate needs to support both native builds and wasm builds, make the wasm support optional and forward it to `orx-parallel`:

```toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false }

[features]
default = []
wasm = ["orx-parallel/wasm"]
```

This is the pattern used by the example computation crates under [examples/wasm/tsp](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/tsp).

The parallel computation code itself does not need a separate wasm-specific branch. The configuration changes; the algorithm code does not.

## What stays the same

The same `orx-parallel` APIs are used in native and wasm builds.

- Keep the computation logic in Rust.
- Keep the parallelization logic in Rust.
- Use the wasm layer only to expose a small API to JavaScript or to a frontend app.

The example projects under `examples/wasm/tsp` follow the same split for React, Yew, Leptos, and a vanilla browser host.

## Required browser-thread setup

Parallel wasm in the browser requires all of the following.

### 1. Build wasm with atomics and shared memory enabled

Your wasm build must target a threaded wasm configuration.

In this repository, the exact wiring is demonstrated in the browser example apps under `examples/wasm/tsp/*/app` and in the matching Rust crates under `examples/wasm/tsp/*/computation` and `examples/wasm/tsp/*/wasm_bindings`.

If the wasm build is not atomics-enabled, the parallel wasm path cannot initialize.

### 2. Serve the app with cross-origin isolation headers

Browser threads require `SharedArrayBuffer`, which in turn requires cross-origin isolation.

In practice, your server must send both headers:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

The Vite-based example apps show this setup.

### 3. Initialize the runtime before the first parallel computation

For the wasm backends, initialization is explicit.

In a typical `wasm_bindings` crate, expose a thin wrapper such as:

```rust
#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}
```

Call and await that initialization before the first parallel execution.

If your frontend uses Web Workers, each worker that performs parallel work must initialize its runtime before its first parallel computation.

## Recommended project structure

The examples in `examples/wasm/tsp` use a four-layer split that scales well:

- `computation/` for pure Rust algorithm code including parallelization by `orx-parallel` api
- `wasm_bindings/` for a thin `wasm_bindgen` boundary
- `components/` or another UI crate for UI logic
- `app/` for the browser host, worker setup, and dev server configuration

This separation keeps the algorithm testable outside the browser and prevents wasm or UI concerns from leaking into the computation crate.

## Dual-build guidance

If your crate should run natively and in the browser:

1. Keep the computation crate free of browser-specific code.
2. Make the wasm feature optional.
3. Add the wasm initialization only in the wasm-facing crate.
4. Keep the browser host responsible for worker startup and HTTP headers.

This is usually the cleanest way to support native tests and benchmarks while also supporting browser threads.

## Troubleshooting

If wasm parallelism does not work as expected, check these first:

- The crate was built with the `wasm` feature enabled.
- The wasm target was built with atomics and shared-memory support.
- The app is served with `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers.
- `init_thread_pool(...)` was called and awaited before the first parallel computation.
- Every worker that executes parallel code initializes its own runtime.

## Example entry points

For complete end-to-end examples, start with one of these:

- `examples/wasm/tsp/react`
- `examples/wasm/tsp/yew`
- `examples/wasm/tsp/leptos`
- `examples/wasm/tsp/vanilla`

Each example uses the same core pattern while adapting the browser host to a different framework.
