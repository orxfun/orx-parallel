# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

High performance, configurable and expressive parallel computing library for computations defined as compositions of iterator methods.

## Parallel Computation by Iterators

Parallel computation is defined using the parallel iterator trait [`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html).

The goal is replacing `iter` with `par`, or `into_iter` with `into_par`, to parallelize the computation defined as a composition of functions through iterators.

The following is an extremely naive [traveling salesperson](https://en.wikipedia.org/wiki/Travelling_salesman_problem) algorithm which randomly generates sequences and picks the one with the minimum duration as the best tour. The example demonstrates chaining of very common and useful `map`, `filter` and `reduce` (`min_by_key` in this example) operations. Notice that the only difference between the sequential and parallel programs is the `par()` call.

```rust
use orx_parallel::*;
use rand::prelude::*;

struct Tour(Vec<usize>);

impl Tour {
    fn new_random(n: usize) -> Self {
        let mut cities: Vec<_> = (0..n).collect();
        cities.shuffle(&mut rand::rng());
        Self(cities)
    }

    fn not_in_standard_order(&self) -> bool {
        self.0.iter().enumerate().any(|(i, c)| i != *c)
    }

    fn duration(&self) -> usize {
        let mut total = 0;
        let links = self.0.iter().zip(self.0.iter().skip(1));
        for (a, b) in links {
            total += (*a as i64 - *b as i64).abs() as usize;
        }
        total
    }
}

let num_tours = 1_000_000;
let num_cities = 10;

// sequential
let best_tour = (0..num_tours)
    .map(|_| Tour::new_random(num_cities))
    .filter(|t| t.not_in_standard_order())
    .min_by_key(|t| t.duration())
    .unwrap();

// parallel
let best_tour = (0..num_tours)
    .par() // parallelized !!
    .map(|_| Tour::new_random(num_cities))
    .filter(|t| t.not_in_standard_order())
    .min_by_key(|t| t.duration())
    .unwrap();
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

All benchmark files are located in [benches](https://github.com/orxfun/orx-parallel/blob/main/benches) directory. The following tables report average execution times of parallel computations in microseconds. The numbers in parentheses represent the ratio of execution time to sequential computation which is used as the baseline (1.00). Parallelized executions of all benchmarks are carried out with default settings. 

Computations are separated into three categories: collect, reduce and early-exit. `ParIter` of **orx-parallel** with default configuration and default parallel runner consistently provides significant improvements in all categories.

### Collect

In this group of benchmark, outputs of the parallel computations are collected into standard vectors.

The arguments of chained transformations such as `filter`, `filter_map` or `map` are test functions applied to the input. Details of the methods can be found in the benchmark **file**s.

> (*) In addition to standard vector, outputs can be collected into a [`SplitVec`](https://crates.io/crates/orx-split-vec) with `ParIter` trait defined in this crate. Collecting into a split vector can provide additional improvements, especially for large outputs, due to avoided copy operations. Note that a split vector provides constant time random access and although it is split to fragments, it asymptotically inherits advantages of contiguous vectors.

| file                                                                                | computation                                                                                                                                       |    sequential |         rayon | orx-parallel | orx-parallel (*) |
|-------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|--------------:|--------------:|-------------:|-----------------:|
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_filter.rs)     | `inputs.into_par()`</br>`.filter(filter).collect()`                                                                                              |   5.92 (1.00) |  12.58 (2.12) |  2.50 (0.42) |  **2.47 (0.42)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_filtermap.rs)  | `inputs.into_par()`</br>`.filter_map(filter_map).collect()`                                                                                      |  15.95 (1.00) |  12.62 (0.79) |  6.75 (0.42) |  **6.37 (0.40)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_flatmap.rs)    | `inputs.into_par()`</br>`.flat_map(flat_map).collect()`                                                                                          | 187.37 (1.00) | 492.18 (2.63) | 57.00 (0.30) | **50.34 (0.27)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_map_filter.rs) | `inputs.into_par()`</br>`.map(map).filter(filter).collect()`                                                                                     |  47.97 (1.00) |  14.69 (0.31) | 11.98 (0.25) | **10.29 (0.21)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_map.rs)        | `inputs.into_par()`</br>`.map(map).collect()`                                                                                                    |  36.21 (1.00) |  14.36 (0.40) | 11.76 (0.32) | **14.47 (0.40)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_long_chain.rs) | `inputs.into_par()`</br>`.map(map1).filter(filter1).map(map2)`</br>`.filter(filter2).map(map3).map(map4)`</br>`.filter(filter4).collect()` |  35.89 (1.00) |  10.11 (0.28) |  7.40 (0.21) |  **6.99 (0.19)** |

### Reduce

In this group, instead of collecting outputs, the results are reduced to a single value. As common examples, `count` and `sum` reductions are used in benchmarks. Additionally, a custom `reduce` function is tested.

> Interestingly, neither *orx-parallel* nor *rayon* can do better than the sequential for the `inputs.into_iter().sum()` computation, they don't even get close. For this computation, sequential iterator seems to be amazingly optimized; for others however, we achieve significant improvements with parallelization.

| file                                                                               | computation                                                                                                                                            |      sequential |           rayon |     orx-parallel |
|------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|----------------:|----------------:|-----------------:|
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/count_filtermap.rs)   | `inputs.into_par()`</br>`.filter_map(filter_map).count()`                                                                                             |    12.67 (1.00) |     9.00 (0.71) |  **3.60 (0.28)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/count_flatmap.rs)     | `inputs.into_par()`</br>`.flat_map(flat_map).count()`                                                                                                 |   217.07 (1.00) |   146.77 (0.68) | **39.87 (0.18)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/count_map_filter.rs)  | `inputs.into_par()`</br>`.map(map).filter(filter).count()`                                                                                            |    51.54 (1.00) |     9.74 (0.19) |  **9.09 (0.18)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/count_map.rs)         | `inputs.into_par()`</br>`.map(map).count()`                                                                                                           |    30.96 (1.00) | **5.46 (0.18)** |      6.43 (0.21) |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_map_filter.rs) | `inputs.into_par()`</br>`  .map(map).filter(filter).reduce(reduce)`                                                                                   |    31.92 (1.00) |    16.16 (0.51) |  **7.07 (0.22)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_map.rs)        | `inputs.into_par()`</br>`.map(map).reduce(reduce)`                                                                                                    |    32.26 (1.00) |     7.85 (0.24) |  **6.96 (0.22)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce.rs)            | `inputs.into_iter().reduce(reduce)`                                                                                                                    |     2.00 (1.00) |    12.18 (6.09) |  **1.09 (0.55)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/sum_filtermap.rs)     | `inputs.into_par()`</br>`.filter_map(filter_map).sum()`                                                                                               |    12.40 (1.00) |     4.66 (0.38) |  **2.88 (0.23)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/sum_flatmap.rs)       | `inputs.into_par_iter()`</br>`.flat_map(flat_map).sum()`                                                                                              |   123.50 (1.00) |    56.49 (0.46) | **21.16 (0.17)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/sum_map_filter.rs)    | `inputs.into_par()`</br>`.map(map).filter(filter).sum()`                                                                                              |     6.22 (1.00) |     4.57 (0.73) |  **1.84 (0.30)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/sum.rs)               | `inputs.into_par().sum()`                                                                                                                              | **0.00 (1.00)** |  9.83 (6449.41) |    0.21 (136.25) |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_long_chain.rs) | `inputs.into_par()`</br>`.map(map1).filter(filter1).map(map2)`</br>`.filter(filter2).map(map3).map(map4)`</br>`.filter(filter4).reduce(reduce)` |    32.28 (1.00) |     8.99 (0.28) |  **6.63 (0.21)** |


### Find

In the last category of computations which allow for *early exit*, `find` method is tested. The element to be found is placed in different positions of the input: in the beginning, in the middle, at the end, and lastly intentionally left out.

| file                                                                             | computation                                                            |    sequential |         rayon |     orx-parallel |
|----------------------------------------------------------------------------------|------------------------------------------------------------------------|--------------:|--------------:|-----------------:|
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/find_flatmap.rs)    | `inputs.into_par()`</br>`.flat_map(flat_map)`</br>`.find(find)`      | 170.80 (1.00) | 120.63 (0.71) | **27.53 (0.16)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/find_map_filter.rs) | `inputs.into_par()`</br>`.map(map).filter(filter)`</br>`.find(find)` |  46.28 (1.00) |  11.96 (0.26) |  **9.67 (0.21)** |
| [⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/find.rs)            | `inputs.into_par().find(find)`                                         |   2.51 (1.00) |  12.15 (4.85) |  **1.24 (0.49)** |

## Contributing

Contributions are welcome! 

Please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR,

* if you notice an error,
* have a question or think something could be improved,
* have an input collection or generator that needs to be parallelized, or
* having trouble representing a particular parallel computation with parallel iterators.

Finally, feel free to contact [me](mailto:orx.ugur.arikan@gmail.com) if you are interested in optimization of the parallel runner, by maybe dynamic adaptation of chunks size decisions, to further improve performance.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
