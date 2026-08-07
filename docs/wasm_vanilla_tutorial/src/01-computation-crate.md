# 01 - Computation Crate

[Previous: 00 - Introduction](00-introduction.md) | [Next: 02 - Wasm Bindings Crate](02-wasm-bindings-crate.md)

**>_** Create the `computation` crate

```bash
cargo new --lib computation
```

**>_** Add dependency to `orx-parallel` in `computation/Cargo.toml`

You may either already add the dependency directly with the `wasm` feature, or keep `wasm` feature optional as demosntrated below.

```toml
[package]
name = "computation"
version = "0.1.0"
edition = "2024"

[dependencies]
orx-parallel = { version = "4.0", default-features = false }

[features]
default = []
wasm = ["orx-parallel/wasm"]
```

**>_** Implement the computation in pure rust

Change contents of `computation/src/lib.rs` as follows:

```rust
use orx_parallel::*;
use std::hint::black_box;

// just a function to simulate an actual computation
fn cpu_mix(x: u64) -> u64 {
    const CPU_MIX_ROUNDS: usize = 500;
    let mut x = black_box(x ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

pub fn compute(input: usize, num_threads: usize) -> u64 {
    (0..input)
        .par() // <= parallelization through orx-parallel
        .num_threads(num_threads)
        .filter(|x| !x.is_multiple_of(42))
        .map(|x| cpu_mix(x as u64))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = compute(3, 8);
        assert_eq!(result, 787191910200961162);
    }
}
```

**>_** Checkpoint

Nothing special here, just check your pure-rust application (*don't mind the overflow error when testing in debug mode*).

```rust
cd computation
cargo test --release
```


***Notes:***

- Keep this crate browser-agnostic, no wasm-bindgen usage here.
- `wasm` feature is optional and forwarded from bindings crate.
- Test the rust logic here.
