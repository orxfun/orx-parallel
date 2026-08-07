# 01 - Computation Crate

[Previous: 00 - Introduction](00-introduction.md) | [Next: 02 - Wasm Bindings Crate](02-wasm-bindings-crate.md)

Create `computation/Cargo.toml`:

```toml
[package]
name = "computation"
version = "0.1.0"
edition = "2024"

[dependencies]
orx-parallel = { path = "../../../../..", default-features = false }

[features]
default = []
wasm = ["orx-parallel/wasm"]
```

Create `computation/src/lib.rs`:

```rust
use orx_parallel::*;

fn fibonacci(n: u64) -> u64 {
    let n = n % 50;
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

pub fn run(input: u32, threads: u32) -> u64 {
    let n = input as u64;

    (0..n)
        .par()
        .num_threads(threads as usize)
        .filter(|x| !x.is_multiple_of(42))
        .map(fibonacci)
        .sum()
}
```

Notes:

- Keep this crate browser-agnostic.
- No wasm-bindgen usage here.
- `wasm` feature is optional and forwarded from bindings crate.
