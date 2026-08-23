# The WASM bindings crate

Create a thin layer to define WebAssembly bindings:

```bash
cargo new --lib wasm_bindings
cd wasm_bindings
```

## Dependencies

We will add dependencies to:

* `wasm-bindgen` for creating WebAssembly bindings,
* to our `computation` crate, again with `wasm` feature.

The `computation` dependency enables the `wasm` feature of `orx-parallel`. That
feature supplies the `init_wasm_parallel_runtime` export used by the worker, so the
application bindings crate does not need to implement the runtime initializer
or depend on `js-sys` directly.

Update `par_wasm/wasm_bindings/Cargo.toml` as follows:

```toml
[package]
name = "wasm_bindings"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
computation = { path = "../computation", features = ["wasm"] }
wasm-bindgen = "0.2"
```

## Exposed functions

Update `par_wasm/wasm_bindings/src/lib.rs` as follows:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn calculate_fibonacci(workload: u32, num_threads: u32) -> u64 {
    computation::calculate_fibonacci(workload as usize, num_threads as usize)
}

#[wasm_bindgen]
pub fn mandelbrot_checksum(limit: u32, num_threads: u32) -> u32 {
    computation::mandelbrot_checksum(limit as usize, num_threads as usize) as u32
}
```

Try building this crate before implementing the frontend:

```bash
RUSTUP_TOOLCHAIN=nightly \
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__heap_base -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
cargo build \
  --target wasm32-unknown-unknown \
  --release \
  -Z build-std=panic_abort,std
```

One level up into `par_wasm` directory:

```bash
cd ..
```
