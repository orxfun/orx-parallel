# WebAssembly Threads Setup

This guide describes how to build `orx-parallel` with web-threaded wasm support.

## Scope

Use this setup when you want parallel execution in browser wasm builds with:

- `wasm32-unknown-unknown` target,
- `wasm-web-threads` feature,
- worker-backed Rayon runtime via `init_thread_pool(...)`.

## Compatibility Matrix

| Build/Runtime | Supported | Notes |
| --- | --- | --- |
| Native targets (`x86_64-*`, etc.) | Yes | Existing behavior unchanged. |
| `wasm32-unknown-unknown` (baseline, no wasm threads feature) | Yes | Compiles with existing CI checks. |
| `wasm32-unknown-unknown` + `wasm-web-threads` + atomics flags | Yes | Intended threaded wasm mode. |
| `wasm32-unknown-unknown` + `wasm-web-threads` without atomics flags | Build may compile, runtime fails fast | Pool usage panics with explicit atomics-required message. |
| wasm-bindgen-test-runner runtime for full web-thread execution | Not guaranteed | Use a thread-ready browser/server setup with COOP/COEP for runtime validation. |

## Runtime Prerequisites

WebAssembly threads require `SharedArrayBuffer`, which in turn requires cross-origin isolation headers.

Typical headers:

- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`

## Toolchain (Nightly)

Thread-enabled wasm builds require nightly and `build-std`.

Recommended baseline:

```toml
[toolchain]
channel = "nightly"
components = ["rust-src"]
targets = ["wasm32-unknown-unknown"]
```

If your Rust distribution supports dated nightlies, you can replace `nightly`
with a pinned date for stricter reproducibility.

## Cargo Config (Recommended)

Create `.cargo/config.toml` in your consuming project:

```toml
[target.wasm32-unknown-unknown]
rustflags = [
  "-C", "target-feature=+atomics,+bulk-memory",
  "-C", "link-arg=--shared-memory",
  "-C", "link-arg=--max-memory=1073741824",
  "-C", "link-arg=--import-memory",
  "-C", "link-arg=--export=__wasm_init_tls",
  "-C", "link-arg=--export=__tls_size",
  "-C", "link-arg=--export=__tls_align",
  "-C", "link-arg=--export=__tls_base",
]

[unstable]
build-std = ["panic_abort", "std"]
```

## Command-Line Alternative

If you do not want to use config files, run:

```bash
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory \
-Clink-arg=--shared-memory -Clink-arg=--max-memory=1073741824 \
-Clink-arg=--import-memory \
-Clink-arg=--export=__wasm_init_tls -Clink-arg=--export=__tls_size \
-Clink-arg=--export=__tls_align -Clink-arg=--export=__tls_base' \
cargo +nightly check \
  --target wasm32-unknown-unknown \
  --features wasm-web-threads \
  -Z build-std=panic_abort,std
```

## Initialization Contract

After wasm module initialization, initialize the thread pool before any parallel call:

1. Initialize wasm module.
2. Call and await `init_thread_pool(n)`.
3. Run parallel computations.

`orx-parallel` fails fast when threaded wasm pool APIs are used before initialization.

## Dual-Build Strategy (Threaded + Fallback)

If you target mixed browser support, ship two wasm builds:

- threaded build (`wasm-web-threads` enabled),
- fallback build (without `wasm-web-threads`).

Choose at runtime using feature detection in JavaScript:

```javascript
import { threads } from "wasm-feature-detect";

let wasmPkg;

if (await threads()) {
  wasmPkg = await import("./pkg-with-threads/index.js");
  await wasmPkg.default();
  await wasmPkg.init_thread_pool(navigator.hardwareConcurrency);
} else {
  wasmPkg = await import("./pkg-without-threads/index.js");
  await wasmPkg.default();
}

// Use exported APIs from whichever build was loaded.
```

## Example

A minimal wasm-threaded example is available at `examples/wasm_web_threads.rs`.

It exposes:

- `init_thread_pool(...)` re-export,
- `parallel_sum(n)` as a parallel computation entry point.

## Smoke Tests

Smoke coverage lives in `tests/wasm_web_threads_smoke.rs` and includes:

- panic path when using `Pool::wasm_web(...)` without initialization,
- success path after awaiting `init_thread_pool(...)`.

Compile smoke tests with the same threaded flags:

```bash
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory \
-Clink-arg=--shared-memory -Clink-arg=--max-memory=1073741824 \
-Clink-arg=--import-memory \
-Clink-arg=--export=__wasm_init_tls -Clink-arg=--export=__tls_size \
-Clink-arg=--export=__tls_align -Clink-arg=--export=__tls_base' \
cargo +nightly test \
  --target wasm32-unknown-unknown \
  --features wasm-web-threads \
  --test wasm_web_threads_smoke \
  -Z build-std=panic_abort,std \
  --no-run
```

## CI Reference

Repository CI includes a dedicated `Check (wasm threads)` job in `.github/workflows/ci.yml` that validates this threaded build path.

## Troubleshooting

### 1) "Wasm web thread pool is not initialized"

Cause:

- Parallel API called before awaiting `init_thread_pool(...)`.

Fix:

1. Initialize wasm module.
2. Await `init_thread_pool(...)`.
3. Only then call parallel operations.

### 2) "Wasm web threading requires atomics-enabled wasm build flags"

Cause:

- Threaded pool used without atomics-enabled build flags.

Fix:

- Build with the flags shown in this document (`+atomics,+bulk-memory` and linker args).

### 3) `wasm-bindgen-test-runner` times out or fails to import worker module

Cause:

- Test runner environment may not match full web worker/thread runtime requirements.

Fix:

- Prefer compile-time smoke checks in CI (`--no-run`) for this path.
- For runtime thread validation, run in a browser/server setup with COOP/COEP.

### 4) `getrandom` backend errors on wasm tests

Cause:

- wasm test/dev dependency graph may require explicit wasm JS backend configuration.

Fix:

- Ensure wasm-target dev dependencies include `getrandom` with `wasm_js` feature.

## Release Notes Draft

Suggested release-note bullets for wasm support:

- Added `wasm-web-threads` feature for web-threaded wasm parallel execution.
- Added `Pool::wasm_web(...)` and `WasmWebPool` for wasm-specific pool integration.
- Added `init_thread_pool(...)` re-export for wasm thread-pool initialization.
- Added fail-fast runtime checks for missing initialization and missing atomics setup.
- Added threaded wasm CI checks and compile-time smoke coverage.
- Added example and docs for wasm threaded build and initialization lifecycle.
