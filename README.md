# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

A high performance, convenient and configurable parallel processing library.

## Parallel Computation by Iterators

Parallel computation is achieved conveniently by the parallel iterator trait [`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html). This allows for changing sequential code that is defined as a composition of functions through iterators into its counterpart by adding one word: `par` or `into_par`.

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
    .into_par() // parallelize with default settings
    .map(compute)
    .filter(select)
    .map(|x| x.0)
    .sum();
assert_eq!(par_result, 286);
```

## Conveniently Configurable

Complexity of distribution of work to parallel threads is boiled down to two straightforward parameters which are easy to reason about:
* [`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.NumThreads.html) represents the degree of parallelization. It can be set to one of the two variants:
  * `Auto`: The library aims to select the best value in order to minimize computation time. Note that in some scenarios where the computation is not sufficiently challenging and does not justify spawning new threads, it will not, and hence, will lead to an efficient use of resources.
  * `Max(n)`: The computation can spawn at most `n` threads. `NumThreads::Max(1)` corresponds to sequential computation.
* [`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.ChunkSize.html) represents the number of elements a parallel worker will pull and process every time it becomes idle. This parameter aims to balance the overhead of parallelization and cost of heterogeneity of tasks. It can be set to one of the three variants:
  * `Auto`: The library aims to select the best value in order to minimize computation time, dynamically adjusting the chunk size for the computation.
  * `Exact(c)`: Chunk sizes will be `c`. This variant gives the control completely to the caller, and hence, suits best to computations to be tuned.
  * `Min(c)`: Chunk sizes will be at least `c`. However, the execution is allowed to pull more elements depending on characteristics of the inputs and used number of threads in order to reduce the impact of parallelization overhead.

```rust
use orx_parallel::*;
use std::num::NonZeroUsize;

let _ = (0..42).par().sum(); // both settings at Auto

let _ = (0..42).par().num_threads(4).sum(); // at most 4 threads
let _ = (0..42).par().num_threads(1).sum(); // sequential
let _ = (0..42).par().num_threads(NumThreads::sequential()).sum(); // also sequential
let _ = (0..42).par().num_threads(0).sum(); // shorthand for NumThreads::Auto

let _ = (0..42).par().chunk_size(16).sum(); // chunks of exactly 16 elements
let c = NonZeroUsize::new(64).unwrap();
let _ = (0..42).par().chunk_size(ChunkSize::Min(c)).sum(); // min 64 elements
let _ = (0..42).par().chunk_size(0).sum(); // shorthand for ChunkSize::Auto

let _ = (0..42).par().num_threads(4).chunk_size(16).sum(); // set both
```

Each computation can be configured independently. This feature is helpful in different ways in different scenarios.

**|> Better Strategy by Problem Knowledge**

Both number of threads and chunk size have `Auto` settings which perform efficiently in general. However, there is no one strategy that performs best for all computations or input characteristics. Sometimes we know better.

For instance, consider a problem where we want to `find` the first feasible or acceptable solution. Assume that finding a solution is expensive. Of course, we want the computation to terminate as soon as possible once a solution is found.
* Setting chunk size to 1 allows for the quickest termination once the solution is found. However, it would have the greatest parallelization overhead.
* Setting a sufficiently large chunk size would have much lower parallelization overhead. However, once a thread finds a solution, the other threads would still need to complete their chunk before termination.

In this scenario where the computation is challenging, parallelization overhead is negligible. Therefore, we could set the chunk size to one (`ChunkSize::Exact(1)`). This trivial strategy indeed turns out to be optimal. A similar example is constructed in [benches/map_find_expensive.rs](https://github.com/orxfun/orx-parallel/blob/main/benches/map_find_expensive.rs).
* `Auto` chunk size settings, as well as, `rayon`'s default settings find the solution in slightly longer time than the sequential computation. This is a sad but possible outcome of a parallel execution.
* On the other hand, simply setting the chunk size to 1, we observe that `ParIter` finds the solution ~15 times faster than the sequential.

**|> Better Strategy by Tuning**

A well known concern in parallelization is the diminishing returns from added resources. Doubling the number of threads usually does not halve the computation time. We usually seek a *sweet spot* where the gain is justified by the allocated resources. Further, as the extreme example above demonstrates, same computation might perform significantly differently for different inputs.

Being able to have complete control on these parameters allows to benchmark and tune performance-critical computations for the relevant inputs on target platforms.

**|> Parallelization in a Concurrent Application**

Consider the following two extreme strategies for an api responding to cpu-heavy computation requests:

* **sequential**: We can handle each request sequentially. Since the api will serve multiple requests concurrently, we could still utilize available resources. However, we could be under-utilizing in low-traffic time intervals. On the other hand, utilizing available resources under high-traffic does not necessarily mean that we are achieving a desired response time.
* **all-in**: We might allow each request to utilize all available resources. Under low-traffic, this could cut down the response time of as much as possible. However, due to the diminishing returns of additional computing resources allocated to one computation, we could be utilizing the resources very inefficiently. We could be worse off due to longer waiting time for resources to become available.

A simple strategy could be to limit the maximum level of parallelization of each computation (`NumThreads::Max`). We can set the limit to an empirical sweet spot that achieves the required response time while still using the resources efficiently. This could already prevent the problems observed in the extreme cases defined above.

A more complicated approach could be to implement a resource allocator which dynamically decides on the number of threads of a request depending on the traffic and utilization of resources at the instant the request arrives.

## Generalization of Sequential and Parallel Computation

Executing a parallel computation with `NumThreads::Max(1)` is equivalent to a sequential computation, without any parallelization overhead. In this sense, `ParIter` is a generalization of sequential and parallel computation.

In order to illustrate, consider the following function which accepts the definition of a computation as a `ParIter`. Note that just as sequential iterators, `ParIter` is lazy. In other words, it is just the definition of the computation. Such a `computation` is passed to the `execute` method together with its settings that can be accessed by `computation.params()`.

However, since the method owns the `computation`, it may decide how to execute (`collect_vec`) it. This implementation will go with the given parallel settings. Unless it is Monday, then it will run sequentially.

```rust
use orx_parallel::*;
use chrono::{Datelike, Local, Weekday};
type Output = String;

fn execute<C: ParIter<Item = Output>>(computation: C) -> Vec<Output> {
    match Local::now().weekday() {
        Weekday::Mon => computation.num_threads(1).collect_vec(),
        _ => computation.collect_vec(),
    }
}
```

## Underlying Approach

This crate has developed as a natural follow up of the [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). You may already find example parallel map, fold and find implementations in the examples. Especially when combined with concurrent collections such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag), implementation of parallel computation has been very straightforward:

* Inputs of the computation are concurrently provided by the concurrent iterator. More precisely, all threads can pull from this iterator whenever they are idle.
* Collection of outputs is a little more involved. The approach depends on whether or not the computation or the caller requires to preserve the order, or on the type of the collection to return. However, above-mentioned concurrent data structures allow for efficient concurrent collection of outputs.

Although this is a simplified version, a parallel map implementation looks similar to the example block below which is easy to reason about.

```rust
use orx_concurrent_iter::*;
use orx_concurrent_bag::*;

fn map(input: u64) -> String {
    input.to_string()
}

fn parallel_map(num_threads: usize, iter: impl ConcurrentIter<Item = u64>) -> SplitVec<String> {
    let outputs = ConcurrentBag::new();
    std::thread::scope(|s| {
        for _ in 0..num_threads {
            s.spawn(|| {
                for output in iter.values().map(map) {
                    outputs.push(output);
                }
            });
        }
    });
    outputs.into_inner()
}
```

## Relation to rayon

See [RelationToRayon](https://github.com/orxfun/orx-parallel/blob/main/docs/RelationToRayon.md) section for a discussion on orx-parallel's similarities and differences from rayon.

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR. The goal of v1 is to allow `ParIter` to cover practical use cases, please open an issue if you have a computation that you cannot express and compute with it. The experimental goal of v2, on the other hand, is to provide a more dynamic and smart parallel executor, please see the related discussion [here](https://github.com/orxfun/orx-parallel/discussions/26).

## License

This library is licensed under MIT license. See LICENSE for details.
