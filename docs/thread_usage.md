# Thread Usage and Pool Configuration

This document explains how `orx-parallel` manages threads and how you can control that behavior at the global and per-computation level.

---

## Global Configuration

There are four ways to use `orx-parallel` depending on which features you enable:

| Feature flags           | Thread pool used         |
| ----------------------- | ------------------------ |
| (default, `std` only)   | `OncePool`               |
| `persistent-pool`       | `BasicPool`              |
| `persistent-pool-rayon` | `rayon_core::ThreadPool` |
| `wasm`                  | `WasmWebPool`            |

### OncePool (default)

`OncePool` is not a persistent thread pool — it does not hold on to any threads between computations. If a computation uses `T` threads, exactly `T` threads are spawned just before the computation starts and released immediately after it completes. When no parallel computation is running, no threads are blocked or held in reserve.

This design fits naturally with `orx-parallel`'s execution model: regardless of the number of tasks in the input, the library always spawns exactly `T` threads. Those `T` threads live for the duration of the computation, continuously pulling tasks from a concurrent task queue, and are all released once the computation completes. Because the number of `thread::spawn` calls is constant — not proportional to input size — the spawning overhead is often insignificant. This makes `OncePool` the default for most use cases.

### BasicPool (`persistent-pool`)

If thread-spawn overhead is measurable in your workload, enable the `persistent-pool` feature to use `BasicPool`. The pool spawns worker threads the first time a parallel computation runs, and those threads are kept alive for the entire lifetime of the application, ready to be reused for every subsequent computation.

The number of worker threads is fixed at startup to the minimum of:
- the value of `ORX_PARALLEL_MAX_NUM_THREADS` environment variable (if set to a positive integer), and
- the available system parallelism (`std::thread::available_parallelism()`).

If neither is available, a fallback of 8 threads is used.

### Rayon Pool (`persistent-pool-rayon`)

For a battle-tested persistent pool with additional features, enable `persistent-pool-rayon`. This uses `rayon_core::ThreadPool` as the backing pool. Thread count follows the same rules as `BasicPool` above.

### WasmWebPool (`wasm`)

When targeting WebAssembly, enable the `wasm` feature to use `WasmWebPool`, which is backed by Web Workers. The setup differs from native targets: you must initialize the pool explicitly (by calling and awaiting the initialization function from JavaScript) before any parallel computation runs.

For a step-by-step guide see [docs/wasm_tutorial](wasm_tutorial/). Working examples are in [examples/wasm/mini](../examples/wasm/) and [examples/wasm/tsp](../examples/wasm/).

### Global thread-count cap

Any pool respects the `ORX_PARALLEL_MAX_NUM_THREADS` environment variable. Set it to a positive integer to impose a hard upper bound on the number of threads any pool will use. When unset or set to `0`, pools are free to use all threads available on the system.

---

## Per-Computation Control

In addition to the global pool configuration, you can tune the thread count for each individual computation:

```rust
// Use at most 4 threads for this computation
inputs.par().num_threads(4).sum()

// Run sequentially (near-zero overhead compared to a plain iterator)
inputs.par().num_threads(1).sum()

// No restriction; use all threads available in the pool (the default)
inputs.par().sum()
```

This matters because the relationship between thread count and throughput is not always linear. Depending on the workload, computation time, and memory pressure, fewer threads can sometimes yield better wall-clock time. `num_threads` lets you tune critical computations independently without changing the global pool.

The actual number of threads used by any computation is always `min(pool_capacity, num_threads_requested, input_size)`.

---

## Pool-Agnostic Design

`orx-parallel` is not tied to any specific thread pool. The [`ParThreadPool`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParThreadPool.html) trait is small and straightforward to implement, so you can plug in any executor. To use a custom pool for a single computation, pass it with `.pool(...)`:

```rust
inputs.par().pool(my_custom_pool).sum()
```
