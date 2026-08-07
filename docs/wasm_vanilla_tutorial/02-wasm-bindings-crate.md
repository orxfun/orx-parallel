# 02 - Wasm Bindings Crate

[Previous: 01 - Computation Crate](01-computation-crate.md) | [Next: 03 - Frontend App](03-frontend-app.md)

Create `wasm_bindings/Cargo.toml`:

```toml
[package]
name = "wasm_bindings"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
computation = { path = "../computation", features = ["wasm"] }
orx-parallel = { path = "../../../../..", default-features = false, features = ["wasm"] }
wasm-bindgen = "0.2"
```

Create `wasm_bindings/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
pub fn run(input: u32, threads: u32) -> u64 {
    computation::run(input, threads)
}
```

Why this layer exists:

- Exposes wasm-safe functions to JS.
- Keeps wasm-specific initialization out of computation logic.
