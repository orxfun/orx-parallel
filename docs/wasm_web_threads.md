# WebAssembly Threads Setup

This guide describes how to build `orx-parallel` with web-threaded wasm support.

## Scope

Use this setup when you want parallel execution in browser wasm builds with:

- `wasm32-unknown-unknown` target,
- `wasm-web-threads` feature,
- worker-backed Rayon runtime via `init_thread_pool(...)`.

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

## CI Reference

Repository CI includes a dedicated `Check (wasm threads)` job in `.github/workflows/ci.yml` that validates this threaded build path.
