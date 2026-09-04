# Using orx-parallel in WebAssembly

This guide explains how to use `orx-parallel` in browser-hosted `wasm32` builds.

If you want to understand the internal runtime design, see [wasm_internals.md](wasm_internals.md).

Live examples:

- TSP demo: https://orx-parallel-wasm-demo-tsp.pages.dev/
- Tutorial: https://orx-parallel-wasm-tutorials.pages.dev/
- Demo and tutorial sources: https://github.com/orxfun/orx-parallel-wasm-demos

## Overview

The documented browser-hosted wasm path uses the `wasm` feature.

- exported pool type: `WasmWebPool`
- exported init function: `init_wasm_parallel_runtime(...)` on atomics-enabled `wasm32`
- implementation: custom worker-backed runtime in `src/pool/pool_impl/wasm_web.rs`

The examples in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos) use this backend.

Browser packaging is provided by the companion `orx-parallel-wasm` crate. It
provides the typed `ParallelWorker` client, the bundler-neutral WASM build and
preparation commands, and integrations for Vite, Webpack, Rspack, and Rollup.

## Which feature to enable

For browser-hosted parallel wasm, use the `wasm` feature.

```toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false, features = ["wasm"] }
```

If your crate needs to build both natively and for the browser, keep the wasm feature optional and forward it:

```toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false }

[features]
default = []
wasm = ["orx-parallel/wasm"]
```

This is the pattern used by the computation crates in the wasm demo repository.

## What stays the same

The main design goal is unchanged:

- keep algorithm code in Rust
- keep parallelization logic in Rust
- keep the wasm layer thin
- let the browser host handle worker startup and serving requirements

In other words, the computation pipeline should usually remain the same between native and wasm builds. The differences are in feature selection, initialization, and host setup.

## Required browser setup

Parallel wasm in the browser requires all of the following.

### 1. Build with atomics and shared memory enabled

The runtime initialization export exists only when all of these hold:

- target is `wasm32`
- the crate feature `wasm` is enabled
- the build includes `target_feature = "atomics"`

The working example apps build with a command of this form:

```bash
RUSTUP_TOOLCHAIN=nightly \
RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__heap_base -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
wasm-pack build ../wasm_bindings --target web --out-dir ../app/pkg -- -Z build-std=panic_abort,std
```

See the app package scripts in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).

If the build is not atomics-enabled, `init_wasm_parallel_runtime(...)` is not available and the parallel runtime cannot be initialized.

### 2. Serve with cross-origin isolation headers

Browser wasm threads require `SharedArrayBuffer`, which in practice means cross-origin isolation.

The host must send:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

See the app server and bundler configuration in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).

Vite dev servers in the examples are configured accordingly. If you serve a production `dist/` directory yourself, your production server must set the same headers.

### 3. Initialize the runtime before the first parallel computation

Initialization is explicit.

In the example `wasm_bindings` crates, the public wrapper looks like this:

```rust
#[wasm_bindgen]
pub fn init_wasm_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_wasm_parallel_runtime(num_threads);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_wasm_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}
```

Call and await this once before the first parallel computation in that wasm runtime.

Notes:

- `num_threads = 0` means automatic thread selection
- `0` uses the crate's resource/env-based auto choice
- calling `init_wasm_parallel_runtime(...)` again with the same thread count resolves successfully
- calling it again with a different thread count is rejected

## Recommended crate structure

The examples use a split that works well in practice:

- `computation/` for the pure Rust algorithm crate
- `wasm_bindings/` for the `wasm_bindgen` boundary
- `app/` for the browser host, worker setup, and dev/prod serving config
- optionally `components/` when the frontend framework benefits from a separate UI crate

This keeps the computation crate testable and reusable outside the browser.

## Minimal example layout

In the TSP examples:

- `computation` depends on `orx-parallel` with `default-features = false`
- `computation` forwards a local `wasm` feature to `orx-parallel/wasm`
- `wasm_bindings` enables that `wasm` feature and exposes a small JS-friendly API
- `app` imports the generated `pkg/wasm_bindings.js` and handles browser orchestration

See the TSP example crates in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).

## `orx-parallel-wasm` integration

The `orx-parallel-wasm` package handles the JavaScript and bundler-specific
parts of a threaded WASM application. Its `build` command runs `wasm-pack`
with the required threaded-WASM flags and prepares the generated package. The
preparation step copies the worker helper beside the generated worker entry,
records the package entry in `orx-parallel-wasm.json`, and prepares the helper
for worker-local WASM initialization with shared memory.

Use the bundler adapter that matches the application:

- `orx-parallel-wasm/vite`
- `orx-parallel-wasm/webpack`
- `orx-parallel-wasm/rspack`
- `orx-parallel-wasm/rollup`

The adapters emit the generated bindings, WASM, and worker assets, rewrite
worker entry paths for the output layout, create stable binding shims, and add
COOP/COEP headers for development or static hosting output. The
The manual vanilla example demonstrates the bundler-neutral API without using
one of these adapters: its `build.mjs` performs the asset
packaging and its `server.mjs` supplies the headers.

The tutorial follows the Vite path for simplicity, then covers the manual build,
other bundlers, and other UI frameworks. The mini examples include vanilla Vite,
a plugin-free vanilla build, React with Vite, Webpack, Rspack, and Rollup.

## Troubleshooting

If parallel wasm does not work as expected, check these first:

- the crate was built for `wasm32`
- the `wasm` feature is enabled
- the wasm build includes atomics and shared-memory flags
- the app is served with COOP/COEP headers
- `init_wasm_parallel_runtime(...)` was awaited before the first parallel run
- you did not attempt to reinitialize with a different thread count
- your build or packaging step preserved the worker helper files used by the selected backend
- the deployed files were rebuilt after updating `orx-parallel-wasm` and do not contain stale generated shims

## Example entry points

For end-to-end working references, start with the TSP demo, tutorial, and mini
examples in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).

All of them follow the same basic rule: initialize once, then run parallel computations through the same Rust API you would use natively.
