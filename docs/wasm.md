# Using orx-parallel in WebAssembly

This guide explains how to use `orx-parallel` in browser-hosted `wasm32` builds.

If you want to understand the internal runtime design, see [wasm_internals.md](wasm_internals.md).

Live examples:

- TSP demo: https://orx-parallel-wasm-demo-tsp.pages.dev/
- tutorials: https://orx-parallel-wasm-tutorials.pages.dev/

## Overview

The documented browser-hosted wasm path uses the `wasm` feature.

- exported pool type: `WasmWebPool`
- exported init function: `init_wasm_thread_pool(...)` on atomics-enabled `wasm32`
- implementation: custom worker-backed runtime in `src/pool/pool_impl/wasm_web.rs`

The examples under `examples/wasm/` use this backend.

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

This is the pattern used by the computation crates under `examples/wasm/tsp/*/computation`.

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

See:

- `examples/wasm/tsp/vanilla/app/package.json`
- `examples/wasm/mini/vanilla_persistent_pool/app/package.json`

If the build is not atomics-enabled, `init_wasm_thread_pool(...)` is not available and the parallel runtime cannot be initialized.

### 2. Serve with cross-origin isolation headers

Browser wasm threads require `SharedArrayBuffer`, which in practice means cross-origin isolation.

The host must send:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

See:

- `examples/wasm/tsp/vanilla/app/vite.config.ts`
- `examples/wasm/mini/vanilla_persistent_pool/app/vite.config.js`
- `examples/wasm/tsp/vanilla/app/scripts/serve-dist.mjs`

Vite dev servers in the examples are configured accordingly. If you serve a production `dist/` directory yourself, your production server must set the same headers.

### 3. Initialize the runtime before the first parallel computation

Initialization is explicit.

In the example `wasm_bindings` crates, the public wrapper looks like this:

```rust
#[wasm_bindgen]
pub fn init_wasm_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_wasm_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_wasm_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}
```

Call and await this once before the first parallel computation in that wasm runtime.

Notes:

- `num_threads = 0` means automatic thread selection
- `0` uses the crate's resource/env-based auto choice
- calling `init_wasm_thread_pool(...)` again with the same thread count resolves successfully
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

See:

- `examples/wasm/tsp/vanilla/computation/Cargo.toml`
- `examples/wasm/tsp/vanilla/wasm_bindings/Cargo.toml`
- `examples/wasm/tsp/vanilla/app/README.md`

## Important note about the `wasm` backend build flow

The custom `wasm` backend uses a JS helper module from `src/pool/pool_impl/wasm_web_start_workers.js`, and the example build scripts include one extra post-build step that copies a worker helper file next to the generated wasm-pack snippet.

That is why the example `build:wasm` scripts do more than just call `wasm-pack build`.

If you are building your own app, the safest path is to start from one of the existing example scripts and adapt it rather than reconstructing the worker bootstrap from scratch.

## Troubleshooting

If parallel wasm does not work as expected, check these first:

- the crate was built for `wasm32`
- the `wasm` feature is enabled
- the wasm build includes atomics and shared-memory flags
- the app is served with COOP/COEP headers
- `init_wasm_thread_pool(...)` was awaited before the first parallel run
- you did not attempt to reinitialize with a different thread count
- your build or packaging step preserved the worker helper files used by the selected backend

## Example entry points

For end-to-end working references, start with:

- `examples/wasm/tsp/vanilla`
- `examples/wasm/tsp/react`
- `examples/wasm/tsp/leptos`
- `examples/wasm/tsp/yew`
- `examples/wasm/mini/vanilla_persistent_pool`
- `examples/wasm/mini/vanilla_temporary_pool`

All of them follow the same basic rule: initialize once, then run parallel computations through the same Rust API you would use natively.
