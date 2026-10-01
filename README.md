# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

High-performance parallel computations with an expressive iterator API.

* [🚀](#performance-and-benchmarks) performant parallelization | [📊 benchmarks](https://orxfun.github.io/orx-parallel-benchmarks/)
* [🛡️](#safe-mutable-per-thread-state) safe per-thread mutable state
* [❓](#first-class-fallible-computation) first-class fallible flows
* [🔁](#recursive-iterators-for-non-linear-data) recursive computation on non-linear data
* [🌐](#wasm-support) WebAssembly support | [▶️ live demo](https://orx-parallel-wasm-demo-tsp.pages.dev/) | [📖 tutorial](https://orx-parallel-wasm-tutorials.pages.dev/)
* [🧩](#runner-strategies-and-extensibility) customizable runner strategies for advanced tuning & experimentation
* [⚙️](#configurable-resource-usage) configurable resource usage

## Parallelization with Iterator Ergonomics

In many pipelines, parallelization is as simple as replacing `iter`, `iter_mut`, `into_iter` with `par`, `par_mut`, `into_par`, respectively.

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
    .map(|_| Tour::random(num_cities)) // ← the rest of the pipeline is unchanged
    .filter(|t| t.starts_at_coffee_shop())
    .min_by_key(|t| t.duration());
```

## What Can Be Parallelized?

### 1. Direct collection support

Built-in parallel iterators are available for common collections and ranges, as well as draining iterators:

* vectors and slices
* `VecDeque`
* ranges
* draining iterators: `par_drain`

### 2. Any arbitrary iterator

Any regular iterator can be parallelized with `iter_into_par()`.

```rust
use orx_parallel::*;

fn par_compute(inputs: impl Iterator<Item = u64>) -> u64 {
    inputs
        .iter_into_par() // ← parallelization over arbitrary iterator
        .filter(|x| !x.is_multiple_of(7))
        .sum()
}

let numbers = vec![4, 2, 9, 14, 1];
assert_eq!(par_compute(numbers.iter().copied()), 16);

let iter = (0u64..100).filter(|x| !x.is_power_of_two());
assert_eq!(par_compute(iter), 4088);
```

This also covers iterators over collections without built-in parallel support, such as maps and sets.

```rust
use orx_parallel::*;
use std::collections::HashMap;

let mut map: HashMap<_, _> = (0..1024).map(|x| (x.to_string(), x)).collect();

map.values_mut()
    .iter_into_par()
    .filter(|x| **x % 2 == 0)
    .for_each(|x| *x *= 2);
```

This generic path works across many iterator sources rather than optimizing for a particular collection. It is especially useful when each task is substantial relative to parallelization overhead.

### 3. Extensible via concurrent iterator abstractions

`orx-parallel` builds on concurrent iterator traits from [`orx-concurrent-iter`](https://github.com/orxfun/orx-concurrent-iter/).
Collections that provide a suitable concurrent iterator implementation, such as `IntoConcurrentIter` or `ConcurrentIterable`, can integrate naturally with `orx-parallel`.

This lets collection-specific parallelization live in the collection's own crate, where its internals are available for optimization. If you need help with a `ConcurrentIter` implementation, please open an issue.

## Performance and Benchmarks

Benchmarks help track practical performance and guide future improvements. Their sources live in a separate repository, where each benchmark can run in isolation and produce accurate measurements, especially when comparing thread pools.

* Live benchmark dashboard: <https://orxfun.github.io/orx-parallel-benchmarks/> with results from the benchmark repository.
* Benchmark sources: <https://github.com/orxfun/orx-parallel-benchmarks>

You can also use the benchmark repository as a starting point for measuring your own computations.

## Safe Mutable Per-Thread State

`use` transformations provide a safe and ergonomic way to use mutable thread-local state in parallel pipelines:

* no unsafe code in application-level iterator logic
* exactly one use-variable per worker thread
* reduced and predictable allocations for stateful workloads

For example, reuse one scratch buffer per worker thread instead of allocating a new `String` for every element:

```rust
use orx_parallel::*;

let words = vec!["Love Rust ", " Hello WORLD", "?"];

// one reusable scratch buffer per thread, instead of allocating for every element
let mut buffers = UseVec::new(|_th_idx| String::new());

let greetings: Vec<String> = words
    .par()
    .use_vec(&mut buffers) // ← mutably lend it to parallel iterator
    .filter_map(|buf, w| {
        buf.clear(); // ← buf: &mut String, reused across elements on this thread
        buf.push_str(w.trim());
        buf.make_ascii_lowercase();
        buf.find(' ')
    })
    .map(|buf, space_idx| buf.chars().skip(space_idx + 1).collect())
    .collect();

assert_eq!(greetings, ["rust", "world"]);
```

For practical use cases, please see [`use_transformation.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/use_transformation.md).

## First-Class Fallible Computation

Fallible parallel flows are a core feature.

* `into_optional()` for `Option<T>` pipelines
* `into_fallible()` for `Result<T, E>` pipelines

After the transformation, the pipeline handles only successful values, much like using `?` in regular Rust code. On failure, it short-circuits and exits early.

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
        .map(|row| parse_qty_and_price(row)) // ← some might return None
        .into_optional() // ← ascend
        .filter(|(qty, _)| *qty >= 2) // ← focus only on success path
        .map(|(qty, unit_price)| qty * unit_price) // ← success path
        .sum()
}

assert_eq!(total_price(&["1,2300", "4,499", "5,1100"]), Some(7496));
assert_eq!(total_price(&["1,2300", "4,???", "5,1100"]), None);
```

## Recursive Iterators for Non-Linear Data

Parallel traversal over recursive structures (such as trees or graphs) is supported out of the box without losing convenient iterator ergonomics.

Even as new work is discovered dynamically, ordered mode enables deterministic traversal: order-sensitive operations follow breadth-first order.

Notice below that after the `par_recursive` call, we use regular iterator methods without additional complexity.

```rust ignore
// provide initial tasks => [root]
// define how to explore new tasks => |node| &node.children
// then use regular parallel iterator API
let result = par_recursive([root], |node| &node.children)
    .map(process_node)
    .reduce(merge_agg);
```

For practical examples, see:

* [`examples/recursive_tree/main.rs`](https://github.com/orxfun/orx-parallel/tree/main/examples/recursive_tree)
* [`examples/recursive_file_system.rs`](https://github.com/orxfun/orx-parallel/blob/main/examples/recursive_file_system.rs)
* [`recursive/tree_collect`](https://github.com/orxfun/orx-parallel-benchmarks/tree/main/recursive/tree_collect)

## WASM Support

`orx-parallel` supports browser-based WebAssembly, with dedicated examples and guides.

* live demo: <https://orx-parallel-wasm-demo-tsp.pages.dev/>
* tutorial: <https://orx-parallel-wasm-tutorials.pages.dev/>
* demo and tutorial sources: <https://github.com/orxfun/orx-parallel-wasm-demos>
* wasm guide: [`docs/wasm.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/wasm.md)
* internals: [`docs/wasm_internals.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/wasm_internals.md)

## Runner Strategies and Extensibility

The [`ParRunner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParRunner.html) trait defines scheduling strategies; select one with `.runner(...)`.

Built-in runners:

* `Runner::adaptive()`: adaptive chunking strategy (default with `std` feature)
* `Runner::fixed()`: pre-computed fixed chunking strategy (default in `no-std` builds)

```rust
use orx_parallel::*; // assume default features used: ["std"]

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

You may also implement your own `ParRunner`, either to tune a specific workload or to explore different scheduling ideas.
For implementation guidance, see [`parallel_runner.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/parallel_runner.md).

## Configurable Resource Usage

`orx-parallel` is not tied to any specific thread pool; it can work with transient threads or persistent thread pools. By default, the library uses the persistent built-in `BasicPool`, which reuses its workers across computations.

Configure the pool with Cargo features and the `ORX_NUM_THREADS` environment variable. When set, the variable limits the number of threads; otherwise, the pool can use all available threads.

```toml
# default: BasicPool (persistent workers, reused across computations)
orx-parallel = { version = "4.0" }

# transient pool: spawn threads, compute, and join for each computation
orx-parallel = { version = "4.0", features = ["transient-pool"] }

# rayon-core pool integration
orx-parallel = { version = "4.0", features = ["persistent-pool-rayon"] }
```

**Pool Selection & Tradeoffs:**

The pool's scheduling strategy is usually less important than the work being performed. `BasicPool` (the default) is suitable for most applications—its workers are created once and kept alive, avoiding the overhead of spawning and joining threads for each parallel computation.

If your application performs only occasional parallel computations and should not retain worker threads between them, enable the `transient-pool` feature. This selects `OncePool`, which spawns the required threads just before a computation and joins them immediately after. The tradeoff is the cost of thread creation and cleanup on each parallel operation.

> Consider a parallel computation of *W* tasks to be executed by *N* threads. The number of thread `spawn` calls in `OncePool` is *N*, regardless of how large *W* is.

You can also tune the thread count for each computation:

```rust
use orx_parallel::*;

let result: Vec<_> = (0..1000)
    .par() // ← can use all threads in the pool
    .map(|x| x * 2)
    .num_threads(4) // ← limit this computation to use <=4 threads
    .collect();

assert_eq!(result.len(), 1000);
```

The [`ThreadPool`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ThreadPool.html) trait is small and straightforward to implement. Since thread pools are independent of runner strategies, you can plug in a custom pool as follows:

```rust,ignore
use orx_parallel::*;

let runner = Runner::adaptive_with_pool(MyPool::new());
let sum = (0..1000)
    .par()
    .runner(runner) // ← using adaptive runner with my pool
    .sum();
```

Please see [`thread_usage.md`](https://github.com/orxfun/orx-parallel/blob/main/docs/thread_usage.md) for detailed information.

**Ad-hoc Parallel Computation:**

The thread pool itself is also exposed directly through [`Pool::global()`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.Pool.html#method.global).

```rust,ignore
use orx_parallel::*;

fn prepare_breakfast(ingredients: &[&str]) {}
fn pack_lunch(ingredients: &[&str]) {}

let ingredients = vec!["apple", "tomato"];

Pool::global().scope(|s| {
    s.run(|| prepare_breakfast(&ingredients));
    s.run(|| pack_lunch(&ingredients));
});

// or

let tasks = tasks![
    || prepare_breakfast(&ingredients),
    || pack_lunch(&ingredients)
];
Pool::global().run_all(tasks);
```

Note that the tasks are not boxed. On the other hand, this approach bypasses the concurrent iterator and runner strategy optimizations that parallel iterators rely on, so it is best suited for a few large, independent tasks rather than many small ones.

### Sequential Execution

Every parallel iterator can also run sequentially on the calling thread:

* use `.num_threads(1)` to keep the parallel pipeline API while disabling parallel execution;
* use `.into_iter()` to consume the pipeline as a regular sequential iterator.

Both options avoid spawning worker threads and avoid using the thread pool.

## Contributing

Contributions are welcome! If you notice an error, have a question, or think something could be improved, please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR.

### Experimental Features

The crate provides an `experimental` feature flag for new capabilities that are actively under development and optimization work. For example, `par_experimental_sort` is a parallel slice sorting implementation currently undergoing evaluation and tuning. Contributions, alternative algorithm designs, performance optimizations, and benchmarks for experimental features are very welcome!

### Research & Runner Development

Parallel runner strategies are open for research and improvement. You can start by looking at the current [`adaptive`](https://github.com/orxfun/orx-parallel/tree/main/src/runner/runner_variants/adaptive_chunk) and [`fixed`](https://github.com/orxfun/orx-parallel/tree/main/src/runner/runner_variants/fixed_chunk) runners, then experiment with a new `ParRunner` implementation.

A useful workflow is to run the tests in this repository and use the [`orx-parallel-benchmarks`](https://github.com/orxfun/orx-parallel-benchmarks) repository to measure the performance impact. Benchmark manifests can point to your own branch; to benchmark your runner as the default, update the `DefaultRunner` alias and `default_runner()` wiring in [`src/runner/mod.rs`](https://github.com/orxfun/orx-parallel/blob/main/src/runner/mod.rs) on that branch. You can also use the benchmark repository as a template for measuring your own specific computation.

### New Parallelizable Collection

If there is an input type or collection you would like to parallelize, please open an issue. Collection-specific support can often be added by implementing the appropriate `ConcurrentIter` integration in the collection crate.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
