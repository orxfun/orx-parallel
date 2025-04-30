# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

High performance, configurable and expressive parallel computing library for computations defined as compositions of iterator methods.

## Parallel Computation by Iterators

Parallel computation is defined using the parallel iterator trait [`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html). This allows for changing sequential code that is defined as a composition of functions through iterators into its parallel counterpart by adding one word: `par` or `into_par`.

```rust
use orx_parallel::*;

struct Input(String);
struct Output(usize);

let compute = |input: Input| Output(input.0.len());
let select = |output: &Output| output.0.is_power_of_two();

let inputs = || (0..1024).map(|x| Input(x.to_string())).collect::<Vec<_>>();

let seq_result: usize = inputs()
    .into_iter()
    .map(compute)
    .filter(select)
    .map(|x| x.0)
    .sum();
assert_eq!(seq_result, 286);

let par_result = inputs()
    .into_par() // parallelize !!
    .map(compute)
    .filter(select)
    .map(|x| x.0)
    .sum();
assert_eq!(par_result, 286);
```

## Configurable

Parallel execution is governed by two straightforward parameters.

* [`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.NumThreads.html) is the degree of parallelization. This is a *capacity parameter* used to limit the resources that can be used by the computation.
  * `Auto`: All available threads can be used, but not necessarily.
  * `Max(n)`: The computation can spawn at most n threads.
  * `Max(1)`: Falls back to sequential execution.
* [`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.ChunkSize.html) represents the number of elements a parallel worker will pull and process every time it becomes idle. This is an *optimization parameter* that can be tuned to balance the overhead of parallelization and cost of heterogeneity of tasks.
  * `Auto`: Let the parallel executor dynamically decide, achieves high performance in general and can be used unless we have useful computation specific knowledge.
  * `Exact(c)`: Chunks will have c elements; gives complete control to the caller. Useful when we have a very good knowledge or want to tune the computation for certain data.
  * `Min(c)`: Chunk will have at least c elements. Parallel executor; however, might decide to pull more if each computation is handled very fast.

When omitted as in the example above, `NumThreads::Auto` and `ChunkSize::Auto` will be used. Configuring parallel computation is **straightforward** and **specific to computation** rather than through a global setting.

```rust
use orx_parallel::*;
use std::num::NonZeroUsize;

let n = 1024;

_ = (0..n).par().sum(); // NumThreads::Auto & ChunkSize::Auto

_ = (0..n).par().num_threads(4).sum(); // <= 4 threads
_ = (0..n).par().num_threads(1).sum(); // sequential
_ = (0..n).par().num_threads(0).sum(); // shorthand for NumThreads::Auto

_ = (0..n).par().chunk_size(64).sum(); // chunks of exactly 64 elements
let c = ChunkSize::Min(NonZeroUsize::new(16).unwrap());
_ = (0..n).par().chunk_size(c).sum(); // chunks of at least 16 elements

_ = (0..n).par().num_threads(4).chunk_size(16).sum(); // set both params
```

Note that `NumThreads::Max(1)` executes the computation sequentially, without any parallelization overhead and benefiting from optimizations of regular [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)s.

This gives the consumer, who actually executes the defined computation, complete control to:

* execute in parallel with the given configuration, or
* execute sequentially, or
* execute in parallel with any number of threads that it decides.

This is guaranteed by the fact that both consuming computation calls and configuration methods require ownership (`self`) of the iterator. In this sense, `ParIter` generalizes computations of sequential computations and parallel executions with different degrees of parallelism.

## Underlying Approach, Extensibility and Parallel Runners

This crates defines parallel computation by combining two basic aspects.

* Pulling **inputs** in parallel is achieved through [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). Concurrent iterator implementations are lock-free, efficient and support pull-by-chunks optimization to reduce the parallelization overhead. A thread can pull any number of inputs from the concurrent iterator every time it becomes idle. This provides the means to dynamically decide on the chunk sizes.
* Writing **outputs** in parallel is handled using thread-safe containers such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag). Similarly, these are lock-free collections that aim for high performance collection of results.

Finally, [`ParallelRunner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParallelRunner.html) trait manages parallelization of the given computation with desired configuration. The objective of the parallel runner is to optimize the chunk sizes to solve the tradeoff between impact of heterogeneity of individual computations and overhead of parallelization.

Inputs of the parallel computations and parallel runners are extensible:

* Any input collection or generator that implements [`IntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIter.html) automatically implements [`IntoParIter`](https://docs.rs/orx-concurrent-iter/latest/orx_parallel/trait.IntoParIter.html), and hence, can be parallelized. Therefore, new collection types can be used by defining their concurrent iterators.
  * *Further, any arbitrary sequential `Iterator` implements [`IterIntoParIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IterIntoConcurrentIter.html) and can be parallelized. Importantly note that this is useful only when computation on pulled elements is not insignificantly small.*
* `ParallelRunner` is a trait and a default implementation is provided in this crate. It is possible to implement and use your *own runner* simply calling [`with_runner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.with_runner) transformation method on the parallel iterator. Default parallel runner targets to be efficient in general. When we have a use case with special characteristics, we can implement a `ParallelRunner` optimized for this scenario and use with the parallel iterators.

## Performance and Benchmarks


# NO MORE


the parallel inputs which are made available by 

This crate has developed as a natural follow up of the [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). You may already find example parallel map, fold and find implementations in the examples. Especially when combined with concurrent collections such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag), implementation of parallel computation has been very straightforward. You may find some details in this [section](https://github.com/orxfun/orx-parallel/blob/main/docs/RelationToRayon.md) and this [discussion](https://github.com/orxfun/orx-parallel/discussions/26).

Benchmarks are tricky, even more in parallel context. Nevertheless, results of [benchmarks](https://github.com/orxfun/orx-parallel/blob/main/benches) defined in this repository are very promising for `Par`. Its performance is often on-par with rayon. It can provide significant improvements in scenarios where the results are collected, such as [map |> filter |> collect](https://github.com/orxfun/orx-parallel/blob/main/benches/map_filter_collect.rs) or [flat_map |> collect](https://github.com/orxfun/orx-parallel/blob/main/benches/flatmap.rs), etc.

## Extensible

## Relation to rayon

See [RelationToRayon](https://github.com/orxfun/orx-parallel/blob/main/docs/RelationToRayon.md) section for a discussion on orx-parallel's similarities and differences from rayon.

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR.

The goal of v1 is to allow `Par` to cover practical use cases, please open an issue if you have a computation that you cannot express and compute with it.

The goal of v2 is to provide a more dynamic and smart parallel executor, please see and join the related discussion [here](https://github.com/orxfun/orx-parallel/discussions/26).

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
