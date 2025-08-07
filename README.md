# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

[High performance](#performance-and-benchmarks), [configurable](#configurable) and [expressive](#parallel-computation-by-iterators) parallel computation library.

## Parallel Computation by Iterators

Parallel computation is defined using the parallel iterator trait [`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html).

The goal is to convert an expressive sequential program into an efficient parallel program only by replacing `iter` with `par`; and `into_iter` with `into_par`.

The following is a naive [traveling salesperson](https://en.wikipedia.org/wiki/Travelling_salesman_problem) algorithm which randomly generates sequences and picks the one with the minimum duration as the best tour. The example demonstrates chaining of very common and useful `map`, `filter` and `reduce` (`min_by_key`) operations. Notice that the only difference between the sequential and parallel programs is the `par()` call.

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
    .map(|_| Tour::random(num_cities))
    .filter(|t| t.not_in_standard_order())
    .min_by_key(|t| t.duration())
    .unwrap();

// parallel
let best_tour = (0..num_tours)
    .par() // parallelized !!
    .map(|_| Tour::random(num_cities))
    .filter(|t| t.not_in_standard_order())
    .min_by_key(|t| t.duration())
    .unwrap();
```

## Parallelizable Collections

Inputs that can be used in parallel computations can be categorized in three groups:

* i. directly parallelizable co<center>llections
* ii. parallelization of any iterator
* iii. parallelization of any collection

### i. Directly Parallelizable Collections

These are collections which are parallelized by utilizing their specific structure to achieve high performance.

This crate provides direct implementations of std collections; the table below lists the most recent table of direct implementations.


| Type | Over References<br>`&T` | Over Mut References <br>`&mut T>` | Over Owned Values<br>` T` |
|:--|:-:|:-:|:-:|
| `v: Vec<T>` | `v.par()` | `v.par_mut()` | `v.into_par()` |
| `v: VecDeque<T>` | `v.par()` | | `v.into_par()` |
| `s: &[T]` | `s.par()`<br>`s.into_par()` | | |
| `s: &mut [T]` | | `s.into_par()` | |
| `r: Range<usize>`| | | `r.par()`<br>`r.into_par()` |

Implementations of custom collections must belong to the respective crates as they most likely require to access the internals. Currently, the following collections are known to allow parallel computation using this crate:

│ [SplitVec](https://crates.io/crates/orx-split-vec) │ [FixedVec](https://crates.io/crates/orx-fixed-vec) │ [LinkedList](https://crates.io/crates/orx-linked-list) │ [Tree](https://crates.io/crates/orx-tree) │ [ImpVec](https://crates.io/crates/orx-imp-vec) │

Since these implementations are particularly optimized for the collection type, it is preferable to start defining parallel computation from the collection whenever available. In other words, for a vector `v`,

* `v.par().map(_).filter(_).reduce(_)` is a better approach than
* `v.iter().iter_into_par().map(_).filter(_).reduce(_)`, which will be explained in the next subsection.

> **extensibility**: Note that any input collection or generator that implements [`IntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIter.html) automatically implements [`IntoParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.IntoParIter.html). Therefore, a new collection can be directly parallelized provided that its concurrent iterator is implemented.

In addition, there exist the following special parallel iterators that can be directly created from the collection.

| Type | Method | Definition |
|---|---|---|
| `v: Vec<T>` | [`v.par_drain(range)`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParallelDrainableOverSlice.html) | Parallel counterpart of `v.drain(range)` |

### ii. Parallelization of Any Iterator

Any arbitrary sequential [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) implements [`IterIntoParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.IterIntoParIter.html) trait and can be converted into a parallel iterator using the `iter_into_par` method.

As demonstrated below, item type of the Iterator can as well be a mutable reference.

```rust
use orx_parallel::*;
use std::collections::HashMap;

let mut map: HashMap<_, _> = (0..1024).map(|x| (x.to_string(), x)).collect();
let par = map.values_mut().iter_into_par(); // mutable parallel iterator from Iterator
par.filter(|x| **x != 42).for_each(|x| *x *= 0);
assert_eq!(map.values().iter_into_par().sum(), 42); // parallel iterator from Iterator
```

This is very powerful since it allows to parallelize all iterables, which includes pretty much every collection and more.

On the other hand, due to being a generic implementation without collection specific optimizations, parallelized computation might underperform its sequential counterpart if the work to be done on each input element is insignificant. For instance, `i` being an arbitrary iterator of numbers, `i.sum()` will most likely be faster than `i.iter_into_par().sum()`.

This being said, `ParIter` takes advantage of certain optimizations, such as buffering and chunk size optimization, in order to improve performance. Therefore, whenever the computation on the iterator elements is more involved than just returning them or adding numbers, we can benefit from parallelization. The respective section of [benchmarks](#parallelization-of-arbitrary-iterators) present significant improvements achieved consistently.

### iii. Parallelization of Any Collection

Lastly, consider a collection which does not provide a direct concurrent iterator implementation. This might be our custom collection, say `MyCollection`; or an external collection without a concurrent iterator implementation, such as the `HashSet<T>`.

There are two methods to parallelize computations over such collections:
* (ii) parallelize using the collection's iterator, or
* (i) collect the elements in a vector and then parallelize the vector.

The following table demonstrates these methods for the `HashSet`; however, they are applicable to any collection with `iter` and `into_iter` methods.

| Type | Method | Over References<br>`&T` | Over Owned Values<br>`T` |
|:--|:-:|---|---|
| `h: HashSet<T>` | ii | `h.iter()`<br>&nbsp;&nbsp;`.iter_into_par()` | `h.into_iter()`<br>&nbsp;&nbsp;`.iter_into_par()` |
|                 | i  | `h.iter()`<br>&nbsp;&nbsp;`.collect::<Vec<_>>()`<br>&nbsp;&nbsp;`.par()` | `h.into_iter()`<br>&nbsp;&nbsp;`.collect::<Vec<_>>()`<br>&nbsp;&nbsp;`.into_par()` |

Note that each approach can be more efficient in different scenarios. As a rule of thumb, the less insignificant the work to be done on elements is, the less critical is the choice, in which case parallelization over iterator (ii) is preferable since it avoids the allocation of the vector.

## Performance and Benchmarks

You may find some sample parallel programs in [examples](https://github.com/orxfun/orx-parallel/blob/main/examples) directory. These examples allow to express parallel computations as iterator method compositions and run quick experiments with different approaches. Examples use `GenericIterator`. As the name suggests, it is a generalization of sequential iterator, rayon's parallel iterator and orx-parallel's parallel iterator, and hence, allows for convenient experiments. You may play with the code, update the tested computations and run these examples by including **generic_iterator** feature, such as:

`cargo run --release --features generic_iterator --example benchmark_collect -- --len 123456 --num-repetitions 10`

Actual benchmark files are located in [benches](https://github.com/orxfun/orx-parallel/blob/main/benches) directory. Tables below report average execution times in microseconds. The numbers in parentheses represent the ratio of execution time to that of sequential computation which is used as the baseline (1.00). Parallelized executions of all benchmarks are carried out with default settings. 

Computations are separated into three categories with respect to how the iterator is consumed: collect, reduce and early-exit. Further, two additional categories are created to test parallelization of arbitrary iterators ([ii](#ii-parallelization-of-any-iterator)) and flexibility in composition of computations.

### Collect

In this group of benchmarks, outputs of parallel computations are collected into vectors. Details of the iterator chains and tested functions can be found in the respective benchmark files (you may use the link in the **file** column).

> **(s)** Outputs can also be collected into a [`SplitVec`](https://crates.io/crates/orx-split-vec), which can provide further improvements by avoiding memory copies. Note that a split vector provides constant time random access; and despite the fact that it is split to fragments, it asymptotically inherits advantages of contiguous vectors.

|file|computation|sequential|rayon|orx-parallel|orx-parallel (s)|
|---|---|---:|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_filter.rs)|`.filter(_).collect()`|2.74 (1.00)|12.14 (4.43)|**1.80 (0.66)**|1.87 (0.68)|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_filtermap.rs)|`.filter_map(_).collect()`|6.96 (1.00)|13.28 (1.91)|3.51 (0.50)|**3.35 (0.48)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_flatmap.rs)|`.flat_map(_).collect()`|77.93 (1.00)|239.83 (3.08)|31.73 (0.41)|**23.79 (0.31)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_map_filter.rs)|`.map(_).filter(_).collect()`|19.24 (1.00)|9.99 (0.52)|6.21 (0.32)|**5.98 (0.31)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_map.rs)|`.map(_).collect()`|18.08 (1.00)|7.98 (0.44)|**5.28 (0.29)**|6.09 (0.34)|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/drain_vec_collect_map_filter.rs)|`.map(_).filter(_).collect()`|19.41 (1.00)|7.54 (0.39)|5.90 (0.30)|**5.77 (0.30)**|


### Reduce

In this group, instead of collecting outputs, the results are reduced to a single value. Some common reductions are `sum`, `count`, `min`, etc.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_map_filter.rs)|`.map(_).filter(_).reduce(_)`|14.15 (1.00)|7.55 (0.53)|**3.86 (0.27)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_map.rs)|`.map(_).reduce(_)`|13.81 (1.00)|6.25 (0.45)|**4.15 (0.30)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce.rs)|`.reduce(_)`|0.97 (1.00)|10.58 (10.91)|**0.90 (0.93)**|


### Find

In this category of computations, computations that allow for *early exit* or *short-circuit* are investigated. As an example, experiments on `find` method are presented; methods such as `find_any`, `any` or `all` lead to similar results.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.flat_map(_).find(_)`|160.24 (1.00)|127.37 (0.79)|**27.66 (0.17)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.map(_).filter(_).find(_)`|43.01 (1.00)|11.14 (0.26)|**8.61 (0.20)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.find(_)`|2.94 (1.00)|12.85 (4.37)|**1.54 (0.52)**|


### Parallelization of Arbitrary Iterators

As discussed in [ii](#ii-parallelization-of-any-iterator), parallelization of regular iterators is a very powerful feature. The benchmarks in this category demonstrate that significant improvements can be achieved provided that the computation on elements is not insignificant. Note that every computation defined after `iter_into_par()` are parallelized; and hence, the work on elements here are the `map` and `filter` computations.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_long_chain.rs)|`…long_chain.collect()`|19.72 (1.00)|32.54 (1.65)|**6.12 (0.31)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_iter_into_par.rs)|`.map(_).filter(_).reduce(_)`|15.17 (1.00)|118.28 (7.80)|**4.98 (0.33)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.map(_).filter(_).find(_)`|42.58 (1.00)|63.60 (1.49)|**7.98 (0.19)**|

### Parallel Mutable Iterators

Finally, we investigate the performance of parallel computation which mutates the input elements. In the benchmarks, we filter elements and update the ones which satisfy the given criterion within the `for_each` call.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/mut_for_each_slice.rs)|`slice.par_mut().filter(_).for_each(_)`|62.61 (1.00)|14.08 (0.22)|**8.45 (0.13)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/mut_for_each_iter.rs)|`iter.iter_into_par().filter(_).for_each(_)`|77.63 (1.00)|78.69 (1.01)|**10.03 (0.13)**|


### Composition

In the final category of benchmarks, impact of long chains of transformations on computation time is tested. You may see such example long chains in the benchmark computations below, where `long_chain.` is a shorthand for `.map(map1).filter(filter1).map(map2).filter(filter2).map(map3).map(map4).filter(filter4)`. Notice that the caller can actually shorten the chains by composing some of them. An obvious one is the `.map(map3).map(map4)` call which could have been one call like `map(map3-then-map4)`. However, this is not always possible as the computation might be conditionally built up in stages.

Nevertheless, the results suggest that the functions are efficiently composed by the parallel iterator.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_long_chain.rs)|`…long_chain.collect()`|14.27 (1.00)|6.33 (0.44)|**3.80 (0.27)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_long_chain.rs)|`…long_chain.reduce(_)`|15.08 (1.00)|6.10 (0.40)|**4.03 (0.27)**|


## Configurable

### Configuration per Computation

Each parallel computation is governed by two main straightforward parameters.

* [`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/enum.NumThreads.html) is the degree of parallelization. This is a *capacity parameter* used to limit the resources that can be used by the computation.
  * `Auto`: All available threads can be used, but not necessarily.
  * `Max(n)`: The computation can spawn at most n threads.
  * `Max(1)`: Falls back to sequential execution on the main thread.
* [`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/enum.ChunkSize.html) represents the number of elements a parallel worker will pull and process every time it becomes idle. This is an *optimization parameter* that can be tuned to balance the overhead of parallelization and cost of heterogeneity of tasks.
  * `Auto`: Let the parallel executor dynamically decide, achieves high performance in general and can be used unless we have useful computation specific knowledge.
  * `Exact(c)`: Chunks will have c elements; gives complete control to the caller. Useful when we have a very good knowledge or want to tune the computation for certain data.
  * `Min(c)`: Chunk will have at least c elements. Parallel executor; however, might decide to pull more if each computation is handled very fast.

See also the last parameter [`IterationOrder`](https://docs.rs/orx-parallel/latest/orx_parallel/enum.IterationOrder.html) with variants `Ordered` (default) and `Arbitrary` which is another useful optimization parameter for specific use cases.

When omitted, `NumThreads::Auto` and `ChunkSize::Auto` will be used. Configuring parallel computation is **straightforward** and **specific to computation** rather than through a global setting.

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

Note that `NumThreads::Max(1)` executes the computation sequentially, without any parallelization overhead and benefiting from optimizations of regular iterators.

This gives the consumer, who actually executes the defined computation, complete control to:

* execute in parallel with the given configuration, or
* execute sequentially, or
* execute in parallel with any number of threads that it decides.

This is guaranteed by the fact that both consuming computation calls and configuration methods require ownership (`self`) of the iterator.

### Global Configuration

Additionally, maximum number of threads that can be used by parallel computations can be globally bounded by the environment variable `ORX_PARALLEL_MAX_NUM_THREADS`. Please see the corresponding [example](https://github.com/orxfun/orx-parallel/blob/main/examples/max_num_threads_config.rs) for details.

## Using Transformation for Mutable Variables

Iterator methods allow us to define expressive computations using closures. These closures are often `FnMut` for sequential iterators allowing to mutably capture variables from the scope. It is clear that this is not possible for parallel iterators due to the fact that it would lead to race condition since multiple threads will have access to the captured solution. Therefore, parallel counterpart of the iterator methods often accept closures implementing `Fn`.

However, it is necessary to have mutable variables for certain programs. A very common example is computations requiring random number generators which are stateful and can create random numbers only with a mutable reference.

[`using`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.using) transformation aims to provide a general and safe solution to this problem as follows:
* One mutable variable per thread; hence, no race conditions.
* The mutable variable is explicitly and mutably accessible by all iterator methods of the parallel iterator obtained by transforming a regular parallel iterator providing the variable to be used.

The following two examples demonstrate the idea and usage; further details can be found in [using.md](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).

```rust ignore
input
    .into_par()
    .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64)) // <-- explicit using
    .map(|_, i| fibonacci((i % 50) + 1) % 100)   // rng: &mut ChaCha20Rng
    .filter(|rng, _: &u64| rng.random_bool(0.4)) // is accessible for
    .map(|rng, i: u64| rng.random_range(0..i))   // all iter methods
    .sum()

let (sender, receiver) = channel();
```

```rust ignore
let (sender, receiver) = channel();
(0..5)
    .into_par()
    .using_clone(sender)
    .for_each(|s, x| s.send(x).unwrap());

let mut res: Vec<_> = receiver.iter().collect();
```



## Underlying Approach and Parallel Runners

This crate defines parallel computation by combining two basic aspects.

* Pulling **inputs** in parallel is achieved through [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). Concurrent iterator implementations are lock-free, efficient and support pull-by-chunks optimization to reduce the parallelization overhead. A thread can pull any number of inputs from the concurrent iterator every time it becomes idle. This provides the means to dynamically decide on the chunk sizes.
* Writing **outputs** in parallel is handled using thread-safe containers such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag). Similarly, these are lock-free collections that aim for high performance collection of results.

Finally, [`ParallelRunner`](https://docs.rs/orx-parallel/latest/orx_parallel/runner/trait.ParallelRunner.html) trait manages parallelization of the given computation with desired configuration. The objective of the parallel runner is to optimize the chunk sizes to solve the tradeoff between impact of heterogeneity of individual computations and overhead of parallelization.

Since it is a trait, parallel runner is customizable. It is possible to implement and use your *own runner* simply by calling [`with_runner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.with_runner) transformation method on the parallel iterator. Default parallel runner targets to be efficient in general. When we have a use case with special characteristics, we can implement a `ParallelRunner` optimized for this scenario and use with the parallel iterators.

## Contributing

Contributions are welcome! 

Please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR,

* if you notice an error,
* have a question or think something could be improved,
* have an input collection or generator that needs to be parallelized, or
* having trouble representing a particular parallel computation with parallel iterators,
* or anything else:)

Finally, feel free to contact [me](mailto:orx.ugur.arikan@gmail.com) if you are interested in optimization of the parallel runner to further improve performance, *by maybe dynamic optimization of chunk size decisions with respect to online collection and analysis of metrics*.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
