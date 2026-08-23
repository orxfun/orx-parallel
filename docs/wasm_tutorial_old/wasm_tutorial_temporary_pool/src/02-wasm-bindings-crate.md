# 02 - Wasm Bindings Crate

[Previous: 01 - Computation Crate](01-computation-crate.md) | [Next: 03 - Frontend App](03-frontend-app.md)


**>_** Create the `wasm_bindings` crate

```bash
cd .. # go back to top level
cargo new --lib wasm_bindings
```

**>_** Add required dependencies and set lib crate type in `wasm_bindings/Cargo.toml`

* this time, we certainly require `wasm` feature of `orx-parallel`
* we also add a dependency to our `computation` crate with `wasm` feature
* finally, we add `js-sys` and `wasm-bindgen` dependencies


```toml
[package]
name = "wasm_bindings"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
computation = { path = "../computation", features = ["wasm"] }
js-sys = "0.3"
orx-parallel = { version = "4.0", default-features = false, features = ["wasm"] }
wasm-bindgen = "0.2"
```

**>_** Define the required wasm bindings in `wasm_bindings/src/lib.rs`

Replace the contents of `lib.rs` with the following code. Note that we expose only the functions that we want to call from the frontend, and additionally the `init_parallel_runtime` function to start up the thread pool.

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// Initializes the shared thread pool used by the parallel computation.
///
/// This function must be called once before invoking `compute`.
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_wasm_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
pub fn compute(input: u32, num_threads: u32) -> u64 {
    computation::compute(input as usize, num_threads as usize)
}
```

**>_** Checkpoint

Run the following command to see if everything works fine:

```bash
cd wasm_bindings
RUSTUP_TOOLCHAIN=nightly \
RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__heap_base -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
cargo build --target wasm32-unknown-unknown -Z build-std=panic_abort,std
```

And also test it out with with `wasm-pack`:

```bash
RUSTUP_TOOLCHAIN=nightly \
RUSTFLAGS='-C target-feature=+atomics -C link-arg=--shared-memory -C link-arg=--max-memory=1073741824 -C link-arg=--import-memory -C link-arg=--export=__heap_base -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
wasm-pack build . --target web --out-dir ../app/pkg -- -Z build-std=panic_abort,std
```

Note that `wasm-pack` will create and locate the `pkg` under `app` which we will create next.

***Notes***

- This create exposes wasm-safe functions to JS, and keeps wasm-specific initialization out of computation logic.
- If you want to test wasm bindings, you may use `wasm_bindgen_test`. Please see the [tests](https://github.com/orxfun/orx-parallel/blob/main/examples/wasm/tsp/vanilla/wasm_bindings/tests/bindings.rs) folder for examples.
