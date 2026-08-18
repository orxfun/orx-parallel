# computation

This crate contains the pure Rust TSP implementation used by the wasm TSP example.

It is intentionally free of wasm-bindgen, JavaScript, and UI concerns, which keeps it easy to test and benchmark as an ordinary Rust crate.

## Responsibilities

- generate TSP instances
- build and improve tours
- run parallel search

## How it enables parallelization

This crate uses `orx-parallel` in `run_search`.

That is enough for native builds, but wasm needs the additional `wasm` feature so the same parallel code can run with browser threads.

There are three common ways to wire `orx-parallel` into a crate:

* Include `orx-parallel` without `wasm` if the crate will never run in wasm.
* Include `orx-parallel` with `wasm` if every build should support wasm threads.
* Make `wasm` optional if the crate should work both in native builds and in wasm builds. This example uses that approach:

```toml
# computation/Cargo.toml
[dependencies]
orx-parallel = { path = "../../../../..", default-features = false }

[features]
default = []
wasm = ["orx-parallel/wasm"]
```

Note that the difference is only in configuration; parallel computation code remains the same.

## How it fits into the example

The `wasm_bindings/` crate exposes the functions from this crate to JavaScript, and `components/` consumes those bindings from the browser (hosted by `app/`).

Note that `wasm_bindings` crate includes the `computation` crate with `wasm` feature:

```toml
# wasm_bindings/Cargo.toml
[dependencies]
computation = { path = "../computation", features = ["wasm"] }
```

