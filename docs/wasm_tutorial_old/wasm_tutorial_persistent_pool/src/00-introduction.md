# 00 - Introduction

[Previous: Tutorial Home](index.md) | [Next: 01 - Computation Crate](01-computation-crate.md)

Goal: a minimal browser app that triggers a parallel Rust computation compiled to wasm.

Architecture:

- `computation/`: pure Rust logic
- `wasm_bindings/`: thin wasm boundary
- `app/`: plain JS + HTML frontend
