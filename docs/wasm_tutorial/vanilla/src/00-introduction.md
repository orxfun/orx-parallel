# Introduction

This tutorial demonstrates how to enable parallel computation expressed as `orx-parallel`s parallel iterator in WebAssembly.

The application in this tutorial is implemented in three parts, each with different responsibilities:

1. `computation/` contains ordinary, testable Rust code; uses `orx-parallel` for parallel computations.
2. `wasm_bindings/` exposes a small JavaScript-facing API.
3. `app/` owns the page and talks to WASM through a worker.

This example uses the `orx-parallel-wasm` package to build the bindings and create the worker.

The app is started with one thread pool. Number of threads in the pool can be capped with `threads: N` when `N` is positive; or the pool is allowed to use all threads when `N` is zero.

Each computation also accepts a thread count:

* `0` uses all initialized threads in the pool,
* while a positive value limits the number of threads that can be assigned to the particular computation.
