# Introduction

The browser cannot call a Rust crate directly. A small threaded-WASM app has three responsibilities:

1. `computation/` contains ordinary, testable Rust code.
2. `wasm_bindings/` exposes a small JavaScript-facing API.
3. `app/` owns the page and talks to WASM through a worker.

This example uses the `orx-parallel-wasm` package to build the bindings and create the worker. The app starts one pool with `threads: 0`, which means automatic sizing. Each computation also accepts a thread count: `0` uses all initialized threads, while a positive value is clamped to the pool capacity.
