# orx-parallel

[![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
[![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)

A high performance, convenient and configurable parallel processing library.

# Features

## Parallel Computation by Iterators

Parallel computation is achieved conveniently by the parallel iterator trait [`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html). This allows for changing sequential code that is defined as a composition of functions through iterators into its counterpart by adding one word.

```rust
use orx_parallel::*;

struct Input(String);
struct Output(usize);

let compute = |input: Input| Output(input.0.len());
let select = |output: &Output| output.0.is_power_of_two();

let seq_result: usize = (0..1024)
    .map(|x| Input(x.to_string()))
    .map(compute)
    .filter(select)
    .map(|x| x.0)
    .sum();
assert_eq!(seq_result, 286);

let par_result = (0..1024)
    .par() // parallelize with default settings
    .map(|x| Input(x.to_string()))
    .map(compute)
    .filter(select)
    .map(|x| x.0)
    .sum();
assert_eq!(par_result, 286);
```

## Easily Configurable

Complexity of distribution of work to parallel threads is boiled down to two straightforward parameters:
* [`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.NumThreads.html) represents the degree of parallelization. It can be one of the two variants:
  * `Auto`: The library aims to select the best value in order to minimize computation time **[a]**.
  * `Max(n)`: The computation can spawn at most `n` threads.
* [`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.ChunkSize.html) represents the number of elements a worker will pull and process every time it becomes idle. This parameter aims to balance the overhead of parallelization and cost of heterogeneity of tasks. It can be one of the three variants:
  * `Auto`: The library aims to select the best value in order to minimize computation time.
  * `Exact(c)`: Chunk sizes will be `c`. This variant gives the control completely to the caller, and hence, suits best to computations to be tuned for specific input set.
  * `Min(c)`: Chunk sizes will be at least `c`. However, the execution is allowed to pull more elements depending on characteristics of the inputs and used number of threads **[b]**.

Rather than globally, each computation can be configured separately. This feature is particularly important when parallel processing is used in an already concurrent environment such as an api serving for cpu-bound computation requests. In such a scenario over-utilization of available CPU resources to gain smaller marginal computation time reductions is likely to be sub-optimal for the overall efficiency of the api. A good idea might be to limit the level of parallelization per request. This value can be tuned depending on the traffic and marginal efficiency gain per added resource to the computation.

```rust
use orx_parallel::*;

let _ = (0..42).par().sum(); // both settings at Auto

let _ = (0..42).par().num_threads(4).sum(); // at most 4 threads
let _ = (0..42).par().num_threads(1).sum(); // sequential
let _ = (0..42).par().num_threads(NumThreads::sequential()).sum(); // also sequential
let _ = (0..42).par().num_threads(0).sum(); // shorthand for NumThreads::Auto

let _ = (0..42).par().chunk_size(16).sum(); // chunks of exactly 16 elements
let c = NonZeroUsize::new(16).unwrap();
let _ = (0..42).par().chunk_size(ChunkSize::Min(c)).sum(); // min 16 elements
let _ = (0..42).par().chunk_size(0).sum(); // shorthand for ChunkSize::Auto

let _ = (0..42).par().num_threads(4).chunk_size(16).sum(); // set both
```

**[a]**: *Note that the parallel execution's objective is to minimize total computation time rather than maximizing the busy time of the threads. Therefore, in some scenarios where the computation is not sufficiently challenging and does not justify spawning new threads, it will not, and hence, will lead to an efficient use of resources.*

**[b]**: *When the computation is not sufficiently challenging and a small chunk size leads to the parallelization overhead to overweigh the actual computation, the execution dynamically increases the chunk size.*


## Generalization of Sequential and Parallel Computation

As the prior example suggests, executing a parallel computation with `NumThreads::Max(1)` is equivalent to a sequential computation. The equivalence is exact, and hence, there exists no parallelization overhead. In this sense, `ParIter` is a generalization of sequential and parallel computation.

In order to illustrate, consider the following function which accepts the definition of a computation as a `ParIter`. Note that just as sequential iterators, `ParIter` is lazy. In other words, it is just the definition of the computation. The `execute` method below receives such a definition of a computation together with its parallelization settings that can be accessed by `computation.params()`.

However, since the method owns the `computation`, it may decide how to execute (`collect_vec`) it. This implementation will go with the given parallel settings. Unless it is Monday, then it will run sequentially.

```rust
use orx_parallel::*;
use chrono::{Datelike, Local, Weekday};

fn execute<C: ParIter<Item = Output>>(computation: C) -> Vec<Output> {
    match Local::now().weekday() {
        Weekday::Mon => computation.num_threads(1).collect_vec(),
        _ => computation.collect_vec(),
    }
}
```

# Some Details on the Underlying Approach

## Relation with concurrent iterators and concurrent collections

This crate has developed as a natural follow up of the [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). You may already find example parallel map, fold and find implementations in the examples. Especially when combined with copy-free concurrent collections such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag), implementation of parallel computation has been very straightforward:

* There exist inputs or tasks to be computed. These inputs are concurrently provided by the concurrent iterator. More precisely, all threads can pull from this iterator whenever they are idle.
* Collection part is a little more involved. The approach depends on whether or not the computation or the caller requires to preserve the order, or on the type of the collection to return. The above-mentioned collections allow for high performance concurrent collection of results.

Once all optimization details are removed, the implementation in the core looks as simple as the parallel map below.

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
## Comparison to rayon

Defining parallel computation through the iterator methods is almost identical in this crate and in rayon. Just as in regular iterators, this is certainly a very nice way to represent the computation.

However, underlying approaches are different. As described in this amazing [blog post](https://smallcultfollowing.com/babysteps/blog/2015/12/18/rayon-data-parallelism-in-rust/), rayon takes a very simple and elegant approach, and builds on top of the `join` primitive. The library is very mature and efficient. I have been able to use rayon in almost all time-critical computation requirements I had so far. Then, why another parallel computation library?
* Because they are different, which is nice.
* It is convenient to have a generalized definition of a computation which can be sequential or parallel to any degree **[c]**.
* It aims to be resource efficient; i.e., it does not use an additional thread if the job is not demanding enough to justify using an additional resource **[d]**.
* It is easily configurable per computation, and hence, conveniently gives control to the caller and allows for tuning computations over relevant sets of inputs.
* Benchmarks are tricky, and trickier in parallel context. At least in many benchmarks defined in this crate, we observe that rayon and orx-parallel perform comparably. There exist certain operations where orx-parallel is able to outperform, such as [`flat_map`](https://github.com/orxfun/orx-parallel/blob/main/benches/flat_map.rs) and [`find`](https://github.com/orxfun/orx-parallel/blob/main/benches/find.rs).
* Although, rayon's primitive is simple to understand, things gets complicated as we move to higher levels, at least for me. On the other hand, in the approach taken here, things are simpler to understand in the high level, as can be seen in the parallel map example above. This simplicity makes it easy to experiment and tune different strategies, hence it is promising to allow for further optimizations.

**[c]**: *One thing that is not super-convenient with rayon is to configure the degree of parallelization. It is possible to set a bound by an environment variable which, I believe, is used to instantiate a global thread pool. However, it is still a global setting rather than a limit on each individual computation. I might be wrong though.*

**[d]**: *An example case could be observed [benches/reduce_sum.rs](https://github.com/orxfun/orx-parallel/blob/main/benches/reduce_sum.rs) where the reduction is nothing but adding numbers. In this benchmark, rayon reduction uses all cores in my pc, while performing only twice faster than the sequential version. Default `ParIter` settings, on the other hand, works with a quarter of the available threads and still performs around 20% faster than rayon. This is a case where the overhead of spawning additional threads overweighs the computation itself.*
