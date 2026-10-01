# Using orx-parallel in WebAssembly

This guide explains how to use `orx-parallel` in browser-hosted `wasm32` builds.

If you want to understand the internal runtime design, see [wasm_internals.md](wasm_internals.md).

Live examples:

- TSP demo: <https://orx-parallel-wasm-demo-tsp.pages.dev/>
- Tutorial: <https://orxfun.github.io/orx-parallel-wasm-demos/>
- Demo and tutorial sources: <https://github.com/orxfun/orx-parallel-wasm-demos>

## Overview

The documented browser-hosted wasm path uses the `wasm` feature.

- exported pool type: `WasmWebPool`
- exported init function: `init_wasm_parallel_runtime(...)` on atomics-enabled `wasm32`
- implementation: custom worker-backed runtime in `src/pools/pool_impl/wasm_web.rs`

The examples in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos) use this backend.

Browser packaging is provided by the companion `orx-parallel-wasm` package. It provides the typed `ParallelWorker` client, the bundler-neutral WASM build and preparation commands, and integrations for Vite, Webpack, Rspack, and Rollup.

## Which feature to enable

For browser-hosted parallel wasm, use the `wasm` feature. If the workload is allocation-heavy and the default allocator becomes a bottleneck, also enable the optional `wasm-allocator` feature.

```toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false, features = ["wasm"] }
```

The allocator integration is opt-in:

```toml
[dependencies]
orx-parallel = {
    version = "4.0",
    default-features = false,
    features = ["wasm", "wasm-allocator"],
}
```

Enabling `wasm-allocator` re-exports `WasmParallelAllocator`; it does not
install a global allocator automatically. Select the shard count in the final WASM crate.

```rust
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static GLOBAL_ALLOCATOR: orx_parallel::WasmParallelAllocator<32> =
    orx_parallel::WasmParallelAllocator::new();
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

The demo package and bundler plugins use `wasm-pack` with a nightly toolchain and target-specific flags. From the demo's `app/` directory, a manual build uses the same settings:

```bash
RUSTUP_TOOLCHAIN=nightly \
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__heap_base -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
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

Initialization is explicit, but the demo apps delegate it to `ParallelWorker.ready()`. The client loads the generated bindings in a
dedicated application worker, runs the bindings' default initializer, then
calls the generated `init_wasm_parallel_runtime(...)` export before accepting computation calls. For example:

```ts
import { ParallelWorker } from "orx-parallel-wasm";
import bindingsUrl from "../pkg/wasm_bindings.js?url";

type Computations = {
    calculate_fibonacci: (workload: number, threads: number) => bigint;
};

const worker = new ParallelWorker<Computations>({
    bindingsUrl,
    methods: ["calculate_fibonacci"],
    threads: 0
});

await worker.ready();
const result = await worker.call("calculate_fibonacci", [50_000, 0]);
```

The `threads` option sets the pool size. In `orx-parallel-wasm`, `threads: 0`
selects `Math.max(1, navigator.hardwareConcurrency ?? 1)`. The `threads`
argument passed to a computation can separately limit that computation; `0`
uses all threads initialized in the pool.

If you invoke generated bindings directly instead of using `ParallelWorker`, run and await the bindings' default initializer and then
`init_wasm_parallel_runtime(num_threads)` before the first parallel computation. For direct initialization, `num_threads = 0` uses the crate's
resource- and environment-based limit. Reinitializing with the same count resolves immediately; requesting a different count rejects the returned `Promise`.

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
- `app` passes the generated bindings URL and allowed methods to `ParallelWorker`, which handles browser-worker startup

See the TSP example crates in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).

## `orx-parallel-wasm` integration

The `orx-parallel-wasm` package handles the JavaScript and bundler-specific
parts of a threaded WASM application. Its build command runs `wasm-pack` with the required threaded-WASM flags. Preparation copies the worker helper beside the generated entry, adjusts it to initialize workers with shared memory, and writes an `orx-parallel-wasm.json` asset manifest.

Use the bundler adapter that matches the application:

- `orx-parallel-wasm/vite`
- `orx-parallel-wasm/webpack`
- `orx-parallel-wasm/rspack`
- `orx-parallel-wasm/rollup`

The adapters include the generated bindings, WASM, and worker assets in the
bundler output and configure development headers where supported. Production hosting must also send the COOP/COEP headers. The manual vanilla example demonstrates the bundler-neutral API without an adapter: its `build.mjs` packages the assets and its `server.mjs` supplies the headers.

The tutorial follows the Vite path for simplicity, then covers the manual build, other bundlers, and other UI frameworks. The mini examples include vanilla Vite, a manual vanilla build, React with Vite, Webpack, Rspack, and Rollup.

## Troubleshooting

If parallel wasm does not work as expected, check these first:

- the crate was built for `wasm32`
- the `wasm` feature is enabled
- the wasm build includes atomics and shared-memory flags
- the app is served with COOP/COEP headers
- `ParallelWorker.ready()` resolved, or direct initialization awaited, before the first parallel run
- you did not attempt to reinitialize with a different thread count
- your build or packaging step preserved the worker helper files used by the selected backend
- the deployed files were rebuilt after updating `orx-parallel-wasm`

## Example entry points

For end-to-end working references, start with the TSP demo, tutorial, and mini examples in [`orx-parallel-wasm-demos`](https://github.com/orxfun/orx-parallel-wasm-demos).

All of them follow the same basic rule: initialize once, then run parallel computations through the same Rust API you would use natively.
