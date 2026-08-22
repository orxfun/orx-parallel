# The computation crate

Keep the algorithm independent from the browser. The example exposes two functions:

```rust
pub fn calculate_fibonacci(workload: usize, num_threads: usize) -> u64
pub fn count_primes(limit: usize, num_threads: usize) -> usize
```

`calculate_fibonacci` maps a range of independent Fibonacci terms across a parallel iterator and sums them. `count_primes` tests candidates in a parallel range and counts the matches. Both functions receive `num_threads` through `.num_threads(...)`, so the UI can compare execution settings without moving algorithm code into TypeScript.

The crate has a `wasm` feature that forwards to `orx-parallel/wasm`:

```toml
[features]
default = []
wasm = ["orx-parallel/wasm"]
```

It can still be tested as a normal Rust crate:

```text
cargo test --manifest-path examples/wasm/mini/vanilla/computation/Cargo.toml
```
