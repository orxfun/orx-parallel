# The computation crate

Keep the algorithm independent from the browser. The example exposes two functions:

```rust
pub fn calculate_fibonacci(workload: usize, num_threads: usize) -> u64
pub fn mandelbrot_checksum(workload: usize, num_threads: usize) -> u64
```

`calculate_fibonacci` maps a range of independent Fibonacci terms across a parallel iterator and sums them. `mandelbrot_checksum` maps a range of points across a parallel iterator, calculates the escape iterations for each point, and sums the results.

Both functions receive `num_threads` through `.num_threads(...)`, so the UI can compare execution settings.

The crate has a `wasm` feature that forwards to `orx-parallel/wasm`:

```toml
[features]
default = []
wasm = ["orx-parallel/wasm"]
```

The crate and parallel computations can be used in regular builds without the `wasm` feature, and the computations can be tested as a normal Rust crate.
