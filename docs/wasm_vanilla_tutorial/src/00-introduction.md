# 00 - Introduction

[Previous: Tutorial Home](index.md) | [Next: 01 - Computation Crate](01-computation-crate.md)

Goal: a minimal browser app that triggers a parallel Rust computation compiled to wasm.

Architecture:

- `computation/`: pure Rust logic
- `wasm_bindings/`: thin wasm boundary
- `app/`: plain JS + HTML frontend

You may access the source code created by following this tutorial in [`examples/wasm/mini/vanilla`](https://github.com/orxfun/orx-parallel/tree/main/examples/wasm/mini/vanilla).
