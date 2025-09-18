# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel crate](https://img.shields.io/crates/d/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

[High performance](#performance-and-benchmarks), [configurable](#configurable) and [expressive](#parallel-computation-by-iterators) parallel computation library.

* [Parallel Computation by Iterators](#parallel-computation-by-iterators)
* [Parallelizable Collections](#parallelizable-collections)
* [Performance and Benchmarks](#performance-and-benchmarks)
* [Fallible Parallel Iterators](#fallible-parallel-iterators)
* [Using Mutable Variables](#using-mutable-variables)
* [Configurations](#configurations)
* [Runner: Pools and Executors](#runner-pools-and-executors)
* [Contributing](#contributing)

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

* i. directly parallelizable collections
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

Implementations of custom collections belong to their respective crates as they most likely require access to internals. Currently, the following collections are known to allow parallel computation using this crate:

│ [SplitVec](https://crates.io/crates/orx-split-vec) │ [FixedVec](https://crates.io/crates/orx-fixed-vec) │ [LinkedList](https://crates.io/crates/orx-linked-list) │ [Tree](https://crates.io/crates/orx-tree) │ [ImpVec](https://crates.io/crates/orx-imp-vec) │

Since these implementations are particularly optimized for the collection type, it is preferable to start defining parallel computation from the collection whenever available. In other words, for a direclty parallelizable collection `col`,

* `col.par().map(_).filter(_).reduce(_)` is a better approach than
* `col.iter().iter_into_par().map(_).filter(_).reduce(_)`, which will be explained in the next subsection.

> **extensibility**: Note that any input collection or generator that implements [`IntoConcurrentIter`](https://docs.rs/orx-concurrent-iter/latest/orx_concurrent_iter/trait.IntoConcurrentIter.html) automatically implements [`IntoParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.IntoParIter.html). Therefore, a new collection can be parallelized provided that its concurrent iterator is implemented.

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

This is powerful since it allows to parallelize all iterables, including pretty much every collection and more.

On the other hand, due to being a generic implementation without collection specific optimizations, parallelized computation might underperform its sequential counterpart if the work to be done on each input element is insignificant. For instance, `i` being an arbitrary iterator of numbers, `i.sum()` will most likely be faster than `i.iter_into_par().sum()`.

This being said, `ParIter` takes advantage of certain optimizations, such as buffering and chunk size optimization, in order to improve performance. Therefore, whenever the computation on the iterator elements is more involved than just returning them or adding numbers, we can benefit from parallelization. The respective section of [benchmarks](#parallelization-of-arbitrary-iterators) present significant improvements achieved consistently.

### iii. Parallelization of Any Collection

Lastly, consider a collection which does not provide a direct concurrent iterator implementation. This might be our custom collection, say `MyCollection`; or an external collection without a concurrent iterator implementation, such as the `HashSet<T>`.

There are two methods to parallelize computations over such collections:
* (ii) parallelize using the collection's iterator, or
* (i) collect the elements in a vector and then parallelize work over the vector.

The following table demonstrates these methods for the `HashSet`; however, they are applicable to any collection with `iter` and `into_iter` methods.

| Type | Method | Over References<br>`&T` | Over Owned Values<br>`T` |
|:--|:-:|---|---|
| `h: HashSet<T>` | ii | `h.iter()`<br>&nbsp;&nbsp;`.iter_into_par()` | `h.into_iter()`<br>&nbsp;&nbsp;`.iter_into_par()` |
|                 | i  | `h.iter()`<br>&nbsp;&nbsp;`.collect::<Vec<_>>()`<br>&nbsp;&nbsp;`.par()` | `h.into_iter()`<br>&nbsp;&nbsp;`.collect::<Vec<_>>()`<br>&nbsp;&nbsp;`.into_par()` |

Note that each approach can be more efficient in different scenarios. For large elements, (ii) might be preferred to avoid allocation of the vector. For insignificant tasks to be performed on each element, (i) might be preferred to take full benefit of vector-specific optimizations.

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
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/drain_vec_collect_map_filter.rs)|`.map(_).filter(_).collect()` [☆][draining_iterator]|19.41 (1.00)|7.54 (0.39)|5.90 (0.30)|**5.77 (0.30)**|

[draining_iterator]: ## "parallel draining iterator"


### Reduce

In this group, instead of collecting outputs, the results are reduced to a single value. Some common reductions are `sum`, `count`, `min`, etc.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_map_filter.rs)|`.map(_).filter(_).reduce(_)`|14.15 (1.00)|7.55 (0.53)|**3.86 (0.27)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_map.rs)|`.map(_).reduce(_)`|13.81 (1.00)|6.25 (0.45)|**4.15 (0.30)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce.rs)|`.reduce(_)`|0.97 (1.00)|10.58 (10.91)|**0.90 (0.93)**|


### Find

Here, computations that allow for *early exit* or *short-circuit* are investigated. As an example, experiments on `find` method are presented; methods such as `find_any`, `any` or `all` lead to similar results.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.flat_map(_).find(_)`|160.24 (1.00)|127.37 (0.79)|**27.66 (0.17)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.map(_).filter(_).find(_)`|43.01 (1.00)|11.14 (0.26)|**8.61 (0.20)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.find(_)`|2.94 (1.00)|12.85 (4.37)|**1.54 (0.52)**|


### Parallelization of Arbitrary Iterators

As discussed in [ii](#ii-parallelization-of-any-iterator), parallelization of regular iterators is a powerful feature. The benchmarks in this category demonstrate that improvements can be achieved provided that the computation on elements is not insignificant.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_long_chain.rs)|`…long_chain.collect()`|19.72 (1.00)|32.54 (1.65)|**6.12 (0.31)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_iter_into_par.rs)|`.map(_).filter(_).reduce(_)`|15.17 (1.00)|118.28 (7.80)|**4.98 (0.33)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/)|`.map(_).filter(_).find(_)`|42.58 (1.00)|63.60 (1.49)|**7.98 (0.19)**|

### Parallel Mutable Iterators

In this group, we investigate the performance of parallel computation which mutates the input elements. In the benchmarks, we filter elements and update the ones which satisfy the given criterion within the `for_each` call.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/mut_for_each_slice.rs)|`slice.par_mut().filter(_).for_each(_)`|62.61 (1.00)|14.08 (0.22)|**8.45 (0.13)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/mut_for_each_iter.rs)|`iter.iter_into_par().filter(_).for_each(_)`|77.63 (1.00)|78.69 (1.01)|**10.03 (0.13)**|


### Composition

In the final category of benchmarks, impact of long chains of transformations on computation time is tested. You may see such example long chains in the benchmark computations below, where `long_chain` is a shorthand for `.map(map1).filter(filter1).map(map2).filter(filter2).map(map3).map(map4).filter(filter4)`. Notice that the caller can actually shorten the chains by composing some of them. An obvious one is the `.map(map3).map(map4)` call which could have been one call like `map(map3-then-map4)`. However, this is not always possible as the computation might be conditionally built up in stages. Further, breaking transformations into smaller pieces help in achieving more descriptive computation definitions.

The results suggest that the functions are efficiently composed by the parallel iterator.

|file|computation|sequential|rayon|orx-parallel|
|---|---|---:|---:|---:|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_long_chain.rs)|`…long_chain.collect()`|14.27 (1.00)|6.33 (0.44)|**3.80 (0.27)**|
|[⇨](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_long_chain.rs)|`…long_chain.reduce(_)`|15.08 (1.00)|6.10 (0.40)|**4.03 (0.27)**|

## Fallible Parallel Iterators

We enjoy rust's [`?`](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator) operator when working with fallible computations. It allows us to focus on and code only the success path. Failure at any step of the computation leads to a short-circuit and immediately returns from the function.

```rust
fn try_to_parse() -> Result<i32, std::num::ParseIntError> {
    let x: i32 = "123".parse()?; // x = 123
    let y: i32 = "24a".parse()?; // returns an Err() immediately
    Ok(x + y)                    // Doesn't run.
}
```

However, we do not have this convenience while working with iterators.

`collect` is the only exception. Normally, it allows us to pick the container to collect the items into.

```rust
let into_vec: Vec<usize> = (0..10).collect();
let into_set: std::collections::HashSet<usize> = (0..10).collect();
```

But it also does something exceptional when the item type is a result:
* The first computation below is similar to above, it simply collects each element to the container which is defined as a vector.
* The second computation; however, is fundamentally different. It collects elements iff all elements are of the Ok variant. Further, it short-circuits the computation as soon as an Err is observed. This is exactly how the `?` operator behaves.

```rust
let into_vec_of_results: Vec<Result<usize, char>> = (0..10).map(|x| Ok(x)).collect();
let into_result_of_vec: Result<Vec<usize>, char> = (0..10).map(|x| Ok(x)).collect();
```

Although convenient, change in the behavior of the collect computation might be considered *unexpected*, at least for me.

Further, we do have not short-circuiting methods for computations other than collect. For instance, it is not as convenient to compute the sum of numbers of an iterator provided that all elements are of the Ok variant, and receive the error otherwise.

In general, the requirement to early exit in fallible computation is common and important both for performance and convenience reasons.

For parallel computation, this crate proposes to explicitly transform an iterator with fallible elements into a fallible parallel iterator.

```rust
use orx_parallel::*;
use std::num::ParseIntError;

let collect: Result<Vec<i32>, ParseIntError> = vec!["7", "2", "34"]
    .into_par()
    .map(|x| x.parse::<i32>())
    .into_fallible_result() // <-- explicit transformation to fallible iterator
    .collect();
```

Currently, there exist two fallible parallel iterators [`ParIterResult`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIterResult.html) and [`ParIterOption`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIterOption.html). The transformation is as follows:

| Regular Iterator | Transformation  Method| Fallible Iterator |
| --- | --- | --- |
| `ParIter<Item=Result<T, E>>` | `into_fallible_result()` | `ParIterResult<Item=T, Error=E>` |
| `ParIter<Item=Option<T>>` | `into_fallible_option()` | `ParIterOption<Item=T>` |

After converting into a fallible iterator, each chaining transformation is based on the success item type. Similar to `?` operator, this allows us to focus on the success path while any error case will be handled by early returning from the iterator with the error.

```rust
use orx_parallel::*;
use std::num::ParseIntError;

let sum: Result<i32, ParseIntError> = vec!["7", "2", "34"]
    .into_par()
    .map(|x| x.parse::<i32>()) // Item = Result<i32, ParseIntError>
    .into_fallible_result() // we are only working with success type after this point
    .map(|x| x + 1)
    .filter(|x| x % 2 == 0)
    .flat_map(|x| [x, x + 1, x + 2])
    .sum(); // returns Result, rather than i32
assert_eq!(sum, Ok(27));

let sum: Result<i32, ParseIntError> = vec!["7", "!!!", "34"]
    .into_par()
    .map(|x| x.parse::<i32>())
    .into_fallible_result()
    .map(|x| x + 1)
    .filter(|x| x % 2 == 0)
    .flat_map(|x| [x, x + 1, x + 2])
    .sum();
assert!(sum.is_err());
```

As demonstrated above, not only `collect` but all computation methods return a `Result`.

To summarize:
* We can use all iterator methods with fallible iterators as well.
* The transformations are based on the success type. All computations return a `Result`:
  * if all computations succeed, it is `Ok` of the value that an infallible iterator would return;
  * it is the first discovered `Err` if any of the computations fails.
* Finally, all computations immediately return in case of an error.

Optional fallible iterator behaves exactly the same, except that `None` is treated as the failure case.

## Using Mutable Variables

Iterator methods allow us to define expressive computations using closures. These closures are often `FnMut` for sequential iterators allowing to mutably capture variables from the scope. It is clear that this is not possible for parallel iterators as it would lead to race condition when multiple threads simultaneously try to access the captured mutable variable. Therefore, parallel counterpart of the iterator methods often accept closures implementing `Fn`.

However, it is necessary to have mutable variables for certain programs. A very common example is computations requiring random number generators which are stateful and can create random numbers only with a mutable reference.

**using** transformation aims to provide a general and safe solution to this problem as follows:
* One mutable variable per thread; hence, no race conditions.
* The mutable variable is explicitly and mutably available to all iterator methods.

The following two examples demonstrate the idea and usage:

* [`using`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.using) takes a closure with thread index as the argument, describing how the mutable variable should be created for each thread.
* [`using_clone`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.using_clone), on the other hand, takes the value to be used as the mutable variable and shares a clone of it with each thread (just a shorthand for `using(|_| sender.clone())`).

In either case, there will exactly be `n` mutable variables created provided that the parallel computation uses `n` threads.

```rust ignore
input
    .into_par()
    .using(|t_idx| ChaCha20Rng::seed_from_u64(42 * t_idx as u64)) // <-- explicit using
    .map(|_, i| fibonacci((i % 50) + 1) % 10)       // rng: &mut ChaCha20Rng
    .filter(|rng, _: &u64| rng.random_bool(0.4))    // is accessible for
    .map(|rng, i: u64| rng.random_range(0..i))      // all iterator methods
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

Further details can be found in [using.md](https://github.com/orxfun/orx-parallel/blob/main/docs/using.md).

## Configurations

### Configuration per Computation

Each parallel computation is governed by two main straightforward parameters.

[`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/enum.NumThreads.html) is the degree of parallelization. This is a *capacity parameter* used to limit the resources that can be used by the computation.
* `Auto`: All available threads can be used, but not necessarily.
* `Max(n)`: The computation can spawn at most n threads.
* `Max(1)`: Falls back to sequential execution on the main thread.

[`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/enum.ChunkSize.html) represents the number of elements a parallel worker will pull and process every time it becomes idle. This is an *optimization parameter* that can be tuned to balance the overhead of parallelization and cost of heterogeneity of tasks.
* `Auto`: Let the parallel executor dynamically decide, achieves high performance in general and can be used unless we have useful computation specific knowledge.
* `Exact(c)`: Chunks will have c elements; gives complete control to the caller. Useful when we have a very good knowledge or want to tune the computation for certain data.
* `Min(c)`: Every chunk will have at least c elements. Parallel executor; however, might decide to pull more if each computation is handled very fast.

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

Note that `NumThreads::Max(1)` executes the computation sequentially.

This gives the consumer, who actually executes the defined computation, complete control to:

* execute in parallel with the given configuration, or
* execute sequentially, or
* execute in parallel with any number of threads that it decides.

This is guaranteed by the fact that both consuming computation calls and configuration methods require ownership (`self`) of the iterator.

### Global Configuration

Additionally, maximum number of threads that can be used by parallel computations can be globally bounded by the environment variable `ORX_PARALLEL_MAX_NUM_THREADS`. Please see the corresponding [example](https://github.com/orxfun/orx-parallel/blob/main/examples/max_num_threads_config.rs) for details.


## Runner: Pools and Executors

This crate defines parallel computation by combining two basic components.

**Pulling inputs**
* Pulling inputs in parallel is achieved through [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). Concurrent iterator implementations are lock-free, efficient and support pull-by-chunks optimization to reduce the parallelization overhead. A thread can pull any number of inputs from the concurrent iterator every time it becomes idle. This provides the means to dynamically decide on the chunk sizes.
* Furthermore, this allows to reduce the overhead of defining creating tasks. To illustrate, provided that the computation will be handled by `n` threads, a closure holding a reference to the input concurrent iterator is defined to represent the computation. This same closure is passed to `n` threads; i.e., `n` spawn calls are made. Each of these threads keep pulling elements from the input until the computation is completed, without requiring to define another task.

**Writing outputs**
* When we collect results, writing outputs is handled using lock-free containers such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag) which aim for high performance collection of results.

There are two main decisions to be taken while executing these components:
* how many threads do we use?
* what is the chunk size; i.e., how many input items does a thread pull each time?

A [`ParallelRunner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParallelRunner) is a combination of a `ParThreadPool` and a `ParallelExecutor` that are responsible for these decisions, respectively.

### ParThreadPool: number of threads

[`ParThreadPool`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParThreadPool) trait generalizes thread pools that can be used for parallel computations. This allows the parallel computation to be generic over thread pools.

When not explicitly set, [`DefaultPool`](https://docs.rs/orx-parallel/latest/orx_parallel/type.DefaultPool) is used:
* When **std** feature is enabled, default pool is the [`StdDefaultPool`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.StdDefaultPool). In other words, all available native threads can be used by the parallel computation. This number can globally bounded by "ORX_PARALLEL_MAX_NUM_THREADS" environment variable when set.
* When working in a **no-std** environment, default pool is the [`SequentialPool`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.SequentialPool). As the name suggests, this pool executes the parallel computation sequentially on the main thread. It can be considered as a placeholder to be overwritten by `with_pool` or `with_runner` methods to achieve parallelism.

*Note that thread pool defines the resource, or upper bound. This upper bound can further be bounded by the [`num_threads`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.num_threads) configuration. Finally, parallel executor might choose not to use all available threads if it decides that the computation is small enough.*

To overwrite the defaults and explicitly set the thread pool to be used for the computation, [`with_pool`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.with_pool) or [`with_runner`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html#tymethod.with_runner) methods are used.

```rust
use orx_parallel::*;

let inputs: Vec<_> = (0..42).collect();

// uses the DefaultPool
// assuming "std" enabled, StdDefaultPool will be used; i.e., native threads
let sum = inputs.par().sum();

// equivalent to:
let sum2 = inputs.par().with_pool(StdDefaultPool::default()).sum();
assert_eq!(sum, sum2);

#[cfg(feature = "scoped_threadpool")]
{
    let mut pool = scoped_threadpool::Pool::new(8);
    // uses the scoped_threadpool::Pool created with 8 threads
    let sum2 = inputs.par().with_pool(&mut pool).sum();
    assert_eq!(sum, sum2);
}

#[cfg(feature = "rayon-core")]
{
    let pool = rayon_core::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();
    // uses the rayon-core::ThreadPool created with 8 threads
    let sum2 = inputs.par().with_pool(&pool).sum();
    assert_eq!(sum, sum2);
}

#[cfg(feature = "yastl")]
{
    let pool = YastlPool::new(8);
    // uses the yastl::Pool created with 8 threads
    let sum2 = inputs.par().with_pool(&pool).sum();
    assert_eq!(sum, sum2);
}
```

`ParThreadPool` implementations of several thread pools are provided in this crate as optional features (see [features](#features) section). Provided that the pool supports scoped computations, it is trivial to implement this trait in most cases (see [implementations](https://github.com/orxfun/orx-parallel/tree/main/src/runner/implementations) for examples).

In most of the cases, *rayon-core*, *scoped_threadpool* and *scoped_pool* perform better than others, and get close to native threads performance with `StdDefaultPool`.

Since parallel computations are generic over the thread pools, performances can be conveniently compared for specific use cases. Such an example benchmark can be found in [collect_filter_map](https://github.com/orxfun/orx-parallel/blob/main/benches/collect_filter_map.rs) file. To have quick tests, you may also use the example [benchmark_pools](https://github.com/orxfun/orx-parallel/blob/main/examples/benchmark_pools.rs).

### ParallelExecutor: chunk size

Once thread pool provides the computation resources, it is [`ParallelExecutor`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParallelExecutor)'s task to distribute work to available threads. As mentioned above, all threads receive exactly the same closure. This closure continues to pull elements from the input concurrent iterator and operate on the inputs until all elements are processed.

The critical decision that parallel executor makes is the chunk size. Depending on the state of the computation, it can dynamically decide on number of elements to pull from the input iterator. The tradeoff it tries to solve is as follows:

* the larger the chunk size,
  * the smaller the parallelization overhead; but also
  * the larger the risk of imbalance in cases of heterogeneity.

## Features

* **std**: This is a **no-std** crate while *std* is included as a default feature. Please use `--no-default-features` flag for no-std use cases. **std** feature enables `StdDefaultPool` as the default thread provider which uses native threads.
* **rayon-core**: This feature enables using `rayon_core::ThreadPool` for parallel computations.
* **scoped_threadpool**: This feature enables using `scoped_threadpool::Pool`.
* **scoped-pool**: This feature enables using `scoped-pool::Pool`.
* **yastl**: This feature enables using `yastl::Pool`.
* **pond**: This feature enables using `pond::Pool`.
* **poolite**: This feature enables using `poolite::Pool`.

## Contributing

Contributions are welcome! 

Please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR,

* if you notice an error,
* have a question or think something could be improved,
* have an input collection or generator that needs to be parallelized,
* want to use a particular thread pool with parallel iterators,
* having trouble representing a particular parallel computation with parallel iterators,
* or anything else:)

Finally, feel free to contact [me](mailto:orx.ugur.arikan@gmail.com) if you are interested in optimization of the parallel runner to further improve performance, *by maybe dynamic optimization of chunk size decisions with respect to online collection and analysis of metrics*.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
