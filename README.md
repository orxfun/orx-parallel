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

`use` transformations provide a safe and ergonomic way to use mutable thread-local state in parallel pipelines.

Highlights:

- convenience and safety: no unsafe code in application-level iterator logic
- memory efficiency: exactly one use-variable per worker thread
- predictable allocation behavior for stateful workloads

```rust
use orx_parallel::*;

struct ThreadData {
    sum: usize,
}

// define how to create thread-local variables
let mut data = UseVec::new(|_th_idx| ThreadData { sum: 0 });

(0..100_000)
    .par() // ← mutably lend it to parallel iterator
    .use_vec(&mut data)
    .for_each(|d, x| d.sum += x); // ← d: &mut ThreadData

let results: Vec<ThreadData> = data.into_vec(); // ← get created vars back
```

For practical use cases, please see [`use_transformation.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/use_transformation.md).

## Configurable Resource Usage

`orx-parallel` is not tied to any specific thread pool; it can work with transient threads or persistent thread pools. The default thread pool can be configured by features and `ORX_PARALLEL_MAX_NUM_THREADS` environment variable.

```toml
# default features -> OncePool
orx-parallel = { version = "4.0" }

# persistent built-in pool
orx-parallel = { version = "4.0", features = ["persistent-pool"] }

# rayon-core pool integration
orx-parallel = { version = "4.0", features = ["persistent-pool-rayon"] }
```

The [`ParThreadPool`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParThreadPool.html) trait is small and straightforward to implement, so you can also plug in any pool with `.pool(...)`.

In addition, you can tune the thread count for each individual computation:

```rust
use orx_parallel::*;

let result: Vec<_> = (0..1000)
	.par() // ← candidate to use all threads in the pool
	.map(|x| x * 2)
	.num_threads(4) // ← limit this computation to use <=4 threads
	.collect();

assert_eq!(result.len(), 1000);
```

Please see [`thread-usage.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/thread_usage.md) for detailed information.


## Runner Strategies and Extensibility

Scheduling is abstracted by [`ParRunner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParRunner.html) and selected with `.runner(...)`.

Built-in runners:

- `Runner::fixed()`: fixed chunking strategy (default in `no-std` builds)
- `Runner::adaptive()`: adaptive chunking strategy (default with `std` feature)

	<!-- .runner(Runner::fixed()) -->
```rust
// default features; i.e., "std" enabled
use orx_parallel::*;

let sum: usize = (0..10_000)
	.par()
	.map(|x| x + 1)
	.sum(); // ← uses adaptive runner by default
assert_eq!(sum, (1..=10_000).sum());

let sum: usize = (0..10_000)
	.par()
    .runner(Runner::fixed()) // ← uses fixed runner
	.map(|x| x + 1)
	.sum();
assert_eq!(sum, (1..=10_000).sum());
```

Alternatively, you may implement your own `ParRunner`:

* which is optimized for a certain type of computations and use it for those computations, or
* which is better than the adaptive or fixed runner in general, and use it everywhere.

This separation makes it easy to:

- tune per-workload execution behavior
- prototype custom runners
- benchmark new scheduling ideas quickly (happy to receive research ideas & contributions)

## Recursive Iterators for Non-Linear Data

Parallel traversal over recursive structures (for example trees) is supported out of the box without losing convenient iterator ergonomics.

Notice below that after the `into_par_recursive` call, we use regular iterator methods without additional complexity.

```rust ignore
[root] // ← we start with initial set of tasks
	.into_par_recursive(|node| &node.children) // ← we define how to explore new tasks
	.map(process_node) // ← we process nodes as if they were in a linear data structure
	.reduce(merge_agg)
	.unwrap_or_default();
```

For practical examples, see:

- [`examples/recursive_tree/main.rs`](https://github.com/orxfun/orx-parallel/tree/main/examples/recursive_tree)
- [`examples/recursive_file_system.rs`](https://github.com/orxfun/orx-parallel/blob/main/examples/recursive_file_system.rs)

## Performance and Benchmarks

The crate is benchmarked extensively with the goal to achieve practical performance and continued improvement.

- Live benchmark dashboard: https://orx-parallel-benchmarks.pages.dev/
- Benchmark sources: [`benches`](https://github.com/orxfun/orx-parallel/tree/main/benches)

## What Can Be Parallelized

### 1. Direct collection support

Common inputs are directly supported, including:

- vectors and slices (`par`, `par_mut`, `into_par`)
- `VecDeque`
- ranges
- draining iterators (`par_drain`)

### 2. Any arbitrary iterator

Any regular iterator can be parallelized with `iter_into_par()`.

```rust
use orx_parallel::*;

// parallelize computation on any iterator
fn par_compute(inputs: impl IntoIterator<Item = u64>) -> u64 {
    inputs
        .into_iter()
        .iter_into_par() // ← parallelize
        .filter(|x| !x.is_multiple_of(7))
        .sum()
}

assert_eq!(par_compute(0..100), 4215);

assert_eq!(par_compute(vec![4, 2, 9, 14, 1]), 16);

let source: Vec<u64> = (0..100).collect();
let iter = source.iter().copied().filter(|x| !x.is_power_of_two());
assert_eq!(par_compute(iter), 4088);
```

This feature allows us to parallelize computations on all iterable collections; on maps or sets, for instance.

```rust
use orx_parallel::*;
use std::collections::HashMap;

let mut map: HashMap<_, _> = (0..1024).map(|x| (x.to_string(), x)).collect();

map.values_mut()
    .iter_into_par()
    .filter(|x| **x % 2 == 0)
    .for_each(|x| *x *= 2);
```

This broad path is generic, rather than being optimized for a specific collection. It works across many iterator sources and is especially useful when each task is substantial relative to parallelization overhead.

### 3. Extensible via concurrent iterator abstractions

`orx-parallel` builds on concurrent iterator traits from `orx-concurrent-iter`.
If a collection provides a suitable concurrent iterator implementation (for example `IntoConcurrentIter` / `ConcurrentIterable`), it can integrate naturally with `orx-parallel`.

In practice, this means collection-specific parallelization can live in the collection crate itself, where internals are available for optimized implementations.

## WASM Support

`orx-parallel` supports browser-hosted wasm with dedicated examples and guides.

- demos (vanilla, React, Leptos, Yew): [`examples/wasm/tsp/`](https://github.com/orxfun/orx-parallel/tree/main/examples/tsp/)
- live demo: https://orx-parallel-wasm-demo-tsp.pages.dev/
- tutorials: https://orx-parallel-wasm-tutorials.pages.dev/
- wasm guide: [`docs/wasm.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/wasm.md)
- internals: [`docs/wasm_internals.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/wasm_internals.md)
