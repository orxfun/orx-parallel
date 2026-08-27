# Thread Usage and Pool Configuration

This document explains how `orx-parallel` manages threads and how you can control that behavior at the global and per-computation level.

---

## Global Configuration

There are four ways to use `orx-parallel` depending on which features you enable:

| Feature flags           | Thread pool used         |
| ----------------------- | ------------------------ |
| default (`std` only)    | `BasicPool`              |
| `transient-pool`        | `OncePool`               |
| `persistent-pool-rayon` | `rayon_core::ThreadPool` |
| `wasm`                  | `WasmWebPool`            |

### BasicPool (default)

`BasicPool` is the default native thread pool when no special pool features are enabled. It creates its worker threads once and keeps them alive for the lifetime of the application, reusing them across parallel computations. This avoids repeated thread-spawn overhead and is a good general-purpose choice for most applications.

The default dependency configuration is:

```toml
orx-parallel = "4.0"
```

The number of worker threads is fixed at startup to the minimum of:
- the value of `ORX_NUM_THREADS` environment variable (if set to a positive integer), and
- the available system parallelism (`std::thread::available_parallelism()`).

If neither is available, a fallback of 8 threads is used.

### OncePool (`transient-pool`)

`OncePool` is not a persistent thread pool — it does not hold on to any threads between computations. If a computation uses `T` threads, exactly `T` threads are spawned just before the computation starts and released immediately after it completes. When no parallel computation is running, no threads are blocked or held in reserve.

This design fits naturally with `orx-parallel`'s execution model: regardless of the number of tasks in the input, the library always spawns exactly `T` threads. Those `T` threads live for the duration of the computation, continuously pulling tasks from a concurrent task queue, and are all released once the computation completes. Because the number of `thread::spawn` calls is constant — not proportional to input size — the spawning overhead is often insignificant.

To use `OncePool`, enable the `transient-pool` feature:

```toml
orx-parallel = { version = "4.0", features = ["transient-pool"] }
```

This is useful for applications with occasional parallel computations that want to create threads, compute, and join them without keeping a persistent worker pool alive between computations. The tradeoff is the cost of spawning threads again for each computation. Pool scheduling is usually less important than the actual work being performed, so `BasicPool` remains a suitable default for most use cases.

### Rayon Pool (`persistent-pool-rayon`)

For a battle-tested persistent pool with additional features, enable `persistent-pool-rayon`. This uses `rayon_core::ThreadPool` as the backing pool. Thread count follows the same rules as `BasicPool` above.

```toml
orx-parallel = { version = "4.0", features = ["persistent-pool-rayon"] }
```

### WasmWebPool (`wasm`)

When targeting WebAssembly, enable the `wasm` feature to use `WasmWebPool`, which is backed by Web Workers. The setup differs from native targets: you must initialize the pool explicitly (by calling and awaiting the initialization function from JavaScript) before any parallel computation runs.

For a step-by-step guide see [docs/wasm_tutorial](wasm_tutorial/). Working examples are in [examples/wasm/mini](../examples/wasm/) and [examples/wasm/tsp](../examples/wasm/).

### Global thread-count cap

Any pool respects the `ORX_NUM_THREADS` environment variable. Set it to a positive integer to impose a hard upper bound on the number of threads any pool will use. When unset or set to `0`, pools are free to use all threads available on the system.

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
