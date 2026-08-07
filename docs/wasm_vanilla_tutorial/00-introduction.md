# 00 - Introduction

[Previous: Tutorial Home](README.md) | [Next: 01 - Computation Crate](01-computation-crate.md)

Goal: a minimal browser app that triggers a parallel Rust computation compiled to wasm.

Computation used in this tutorial:

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

let n = input as u64;
let result = (0..n)
    .par()
    .filter(|x| !x.is_multiple_of(42))
    .map(fibonacci)
    .sum();
```

Architecture:

- `computation/`: pure Rust logic
- `wasm_bindings/`: thin wasm boundary
- `app/`: plain JS + HTML frontend

Suggested location (parallel to other wasm examples):

```text
examples/wasm/fib/vanilla/
  computation/
  wasm_bindings/
  app/
```
