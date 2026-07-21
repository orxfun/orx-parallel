# computation

This crate contains the pure Rust TSP implementation used by the wasm vanilla example.

It is intentionally free of wasm-bindgen, JavaScript, and UI concerns.

This simplifies maintenance of the crate; it can be tested (`cargo test`) or benchmarked (`cargo bench`), etc., normally as a pure rust crate.

## Responsibilities

- generate TSP instances
- build and improve tours
- run sequential and parallel search strategies

## How it enables parallelization

This crate enables parallelization by default using `orx-parallel` within the exposed `run_search_parallel` function.

However, this is not sufficient to enable parallelization within wasm.

The crate introduces the `wasm-web-threads` feature which in turn activates `orx-parallel/wasm-web-threads` feature, and this enables parallel computation in wasm.

Note that there are the following three design alternatives for a crate using parallel computation via `orx-parallel`:

* Include `orx-parallel` without `wasm-web-threads` feature if we do not intend to use it in wasm. 
* Include `orx-parallel` with `wasm-web-threads` feature if we always want to use it in wasm.
* Include `orx-parallel` with optionally adding `wasm-web-threads` feature if we want to allow using the library both in wasm or in different builds. This is demonstrated in this example in Cargo.toml as follows:

```toml
# computation/Cargo.toml
[dependencies]
orx-parallel = { version = "4.0", default-features = false }

[features]
default = []
wasm-web-threads = ["orx-parallel/wasm-web-threads"]
```

Note that the change is only in configuration, parallel computation code remains the same.

## How it fits into the example

The `wasm_bindings/` crate exposes the functions from this crate to JavaScript, and `ui/` consumes those bindings from the browser.

Note that `wasm_bindings` crate includes the `computation` crate with `wasm-web-threads` feature:

```toml
# wasm_bindings/Cargo.toml
[dependencies]
computation = { path = "../computation", features = ["wasm-web-threads"] }
```

