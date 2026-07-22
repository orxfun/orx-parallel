# computation

This crate contains the pure Rust TSP implementation used by the wasm vanilla example.

It is intentionally free of wasm-bindgen, JavaScript, and UI concerns, which keeps it easy to test and benchmark as an ordinary Rust crate.

## Responsibilities

- generate TSP instances
- build and improve tours
- run sequential and parallel search strategies

## How it enables parallelization

This crate uses `orx-parallel` in `run_search_parallel`.

That is enough for native builds, but wasm needs the additional `wasm-web-threads` feature so the same parallel code can run with browser threads.

There are three common ways to wire `orx-parallel` into a crate:

* Include `orx-parallel` without `wasm-web-threads` if the crate will never run in wasm.
* Include `orx-parallel` with `wasm-web-threads` if every build should support wasm threads.
* Make `wasm-web-threads` optional if the crate should work both in native builds and in wasm builds. This example uses that approach:

```toml
# computation/Cargo.toml
[dependencies]
orx-parallel = { path = "../../../../..", default-features = false }

[features]
default = []
wasm-web-threads = ["orx-parallel/wasm-web-threads"]
```

Note that the difference is only in configuration; parallel computation code remains the same.

## How it fits into the example

The `wasm_bindings/` crate exposes the functions from this crate to JavaScript, and `ui/` consumes those bindings from the browser.

Note that `wasm_bindings` crate includes the `computation` crate with `wasm-web-threads` feature:

```toml
# wasm_bindings/Cargo.toml
[dependencies]
computation = { path = "../computation", features = ["wasm-web-threads"] }
```

