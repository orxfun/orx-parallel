# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

High-performance, configurable, expressive parallel computations with an iterator-style API.

## Install

```toml
[dependencies]
orx-parallel = "4.0"
```

## Parallelization with Iterator Ergonomics

In many pipelines, parallelization is as simple as **`iter → par`** and **`into_iter → into_par`** substitution.

```rust
use orx_parallel::*;
use rand::prelude::*;

struct Tour(Vec<usize>);

impl Tour {
    fn random(n: usize) -> Self {
        let mut cities: Vec<_> = (0..n).collect();
        cities.shuffle(&mut rand::rng());
        Self(cities)
    }

    fn starts_at_coffee_shop(&self) -> bool {
        self.0.first() == Some(&7)
    }

    fn duration(&self) -> u64 {
        let links = self.0.iter().zip(self.0.iter().skip(1));
        links
            .map(|(a, b)| (*a as i64 - *b as i64).unsigned_abs())
            .sum::<u64>()
    }
}

let num_tours = 1_000_000;
let num_cities = 10;

// sequential
let best_tour = (0..num_tours)
    .map(|_| Tour::random(num_cities))
    .filter(|t| t.starts_at_coffee_shop())
    .min_by_key(|t| t.duration());

// parallel
let best_tour = (0..num_tours)
    .par() // ← parallelized
    .map(|_| Tour::random(num_cities))
    .filter(|t| t.starts_at_coffee_shop())
    .min_by_key(|t| t.duration());
```

## First-Class Fallible Computation

Fallible parallel flows are a core feature.

- `into_optional()` for `Option<T>` pipelines
- `into_fallible()` for `Result<T, E>` pipelines

After the transformation, you continue writing only the success path, similar in spirit to using `?` in regular Rust code. Any failure short-circuits with early exit.

```rust
use orx_parallel::*;

fn parse_qty_and_price(row: &str) -> Option<(u64, u64)> {
    let mut parts = row.split(',');
    let qty = parts.next()?.parse::<u64>().ok()?;
    let unit_price = parts.next()?.parse::<u64>().ok()?;
    Some((qty, unit_price))
}

fn total_price(rows: &[&str]) -> Option<u64> {
    rows.par()
        .map(|row| parse_qty_and_price(row)) // ← might fail
        .into_optional() // ← uplift & focus on success path
        .filter(|(qty, _)| *qty >= 2)
        .map(|(qty, unit_price)| qty * unit_price)
        .sum()
}

assert_eq!(total_price(&["1,2300", "4,499", "5,1100"]), Some(7496));
assert_eq!(total_price(&["1,2300", "4,???", "5,1100"]), None);
```

## Use Transformations: Safe Mutable Per-Thread State

`use` transformations provide a safe and ergonomic way to use mutable worker-local state in parallel pipelines.

Highlights:

- convenience and safety: no unsafe code in application-level iterator logic
- memory efficiency: exactly one use-variable per worker thread
- predictable allocation behavior for stateful workloads

TODO


You can pre-create one state value per worker with `UseVec` and pass it with `use_vec`, or construct state per thread with `using`/`using_clone`.

```rust ignore
use orx_parallel::*;

struct ThreadData {
	sum: usize,
}

let mut data = UseVec::new(|_| ThreadData { sum: 0 });

(0..100_000)
	.into_par()
	.use_vec(&mut data)
	.for_each(|d, x| d.sum += x);
```

For a practical memory-allocation comparison and chart:

- `examples/use_impact_on_memory/README.md`

## Configurable Resource Usage

Thread usage is controlled by three layers:

1. pool capacity
2. global environment limit via `ORX_PARALLEL_MAX_NUM_THREADS`
3. per-computation `.num_threads(...)`

The actual thread count is the minimum of the active limits.

```rust ignore
use orx_parallel::*;

let result: Vec<_> = (0..1000)
	.into_par()
	.map(|x| x * 2)
	.num_threads(4)
	.collect();

assert_eq!(result.len(), 1000);
```

Example and details:

- `examples/max_num_threads_config.rs`
- `docs/threading_model.md`

## Thread-Pool Agnostic

`orx-parallel` abstracts pool execution behind `ParThreadPool`.

Built-in pool options:

- `OncePool`: lightweight default, spawns up to `T` worker threads per computation and releases them afterward
- `BasicPool`: persistent pool for repeated computations
- `rayon_core::ThreadPool`: supported when `persistent-pool-rayon` is enabled

Feature selection:

```toml
# default features -> OncePool
orx-parallel = { version = "4.0" }

# persistent built-in pool
orx-parallel = { version = "4.0", features = ["persistent-pool"] }

# rayon-core pool integration
orx-parallel = { version = "4.0", features = ["persistent-pool-rayon"] }
```

You can also implement your own `ParThreadPool` and attach it with `.pool(...)`.

## Runner Strategies and Extensibility

Scheduling is abstracted by `ParRunner` and selected with `.runner(...)`.

Built-in runners:

- `Runner::fixed()`: fixed chunking strategy
- `Runner::adaptive()`: adaptive chunking strategy (default when `std` is enabled)

```rust ignore
use orx_parallel::*;

let sum: usize = (0..10_000)
	.par()
	.runner(Runner::fixed())
	.map(|x| x + 1)
	.sum();

assert_eq!(sum, (1..=10_000).sum());
```

This separation makes it easy to:

- tune per-workload execution behavior
- prototype custom runners
- benchmark new scheduling ideas quickly

## Recursive Iterators for Non-Linear Data

Parallel traversal over recursive structures (for example trees) is supported out of the box.

```rust ignore
[root]
	.into_par_recursive(|node| &node.children)
	.num_threads(num_threads)
	.map(process_node)
	.reduce(merge_agg)
	.unwrap_or_default();
```

See:

- `examples/recursive_file_system.rs`
- `examples/recursive_tree/main.rs`

## Performance and Benchmarks

The crate is benchmarked extensively, including comparisons with Rayon.

- Live benchmark dashboard: https://orx-parallel-benchmarks.pages.dev/
- Benchmark sources: `benches/`

The focus is practical performance and continued improvement through measurable iteration.

## What Can Be Parallelized

### 1. Direct collection support

Common inputs are directly supported, including:

- vectors and slices (`par`, `par_mut`, `into_par`)
- `VecDeque`
- ranges
- draining iterators (`par_drain`)

### 2. Any arbitrary iterator

Any regular iterator can be parallelized with `iter_into_par()`.

```rust ignore
use orx_parallel::*;
use std::collections::HashMap;

let mut map: HashMap<_, _> = (0..1024).map(|x| (x.to_string(), x)).collect();

map.values_mut()
	.iter_into_par()
	.filter(|x| **x % 2 == 0)
	.for_each(|x| *x *= 2);
```

This broad path is intentionally generic: it works across many iterator sources and is especially useful when each task is substantial relative to parallelization overhead.

### 3. Extensible via concurrent-iterator abstractions

`orx-parallel` builds on concurrent-iterator traits from `orx-concurrent-iter`.
If a collection provides a suitable concurrent iterator implementation (for example `IntoConcurrentIter` / `ConcurrentIterable`), it can integrate naturally with `orx-parallel`.

In practice, this means collection-specific parallelization can live in the collection crate itself, where internals are available for optimized implementations.

## WASM Support

`orx-parallel` supports browser-hosted wasm with dedicated examples and guides.

- demos (vanilla, React, Leptos, Yew): `examples/wasm/tsp/`
- live demo: https://orx-parallel-wasm-demo-tsp.pages.dev/
- tutorials: https://orx-parallel-wasm-tutorials.pages.dev/
- wasm guide: `docs/wasm.md`
- internals: `docs/wasm_internals.md`

The objective on web targets is the same: keep parallelization convenient, explicit, and safe.

## Further Reading

- API docs: https://docs.rs/orx-parallel
- threading model details: `docs/threading_model.md`
- use transformation details: `docs/using.md`

