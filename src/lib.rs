//! # orx-parallel
//!
//! [![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
//! [![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)
//!
//! A performant and configurable parallel computing library for computations defined as composition of iterator methods.
//!
//! ## Parallel Computation by Iterators
//!
//! Parallel computation is achieved conveniently by the parallel iterator trait [`Par`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.Par.html). This allows for changing sequential code that is defined as a composition of functions through iterators into its counterpart by adding one word: `par` or `into_par`.
//!
//! ```rust
//! use orx_parallel::*;
//!
//! struct Input(String);
//! struct Output(usize);
//!
//! let compute = |input: Input| Output(input.0.len());
//! let select = |output: &Output| output.0.is_power_of_two();
//!
//! let inputs = || (0..1024).map(|x| Input(x.to_string())).collect::<Vec<_>>();
//!
//! let seq_result: usize = inputs()
//!     .into_iter()
//!     .map(compute)
//!     .filter(select)
//!     .map(|x| x.0)
//!     .sum();
//! assert_eq!(seq_result, 286);
//!
//! let par_result = inputs()
//!     .into_par() // parallelize with default settings
//!     .map(compute)
//!     .filter(select)
//!     .map(|x| x.0)
//!     .sum();
//! assert_eq!(par_result, 286);
//! ```
//!
//! Below code block includes some basic examples demonstrating different sources providing references or values as inputs of the parallel computation.
//!
//! ```rust
//! use orx_parallel::*;
//! use std::collections::*;
//!
//! fn test<I: Par<Item = usize>>(iter: I) {
//!     let result = iter.filter(|x| x % 2 == 1).map(|x| x + 1).sum();
//!     assert_eq!(6, result);
//! }
//!
//! let range = 1..4;
//! test(range.par());
//!
//! let vec = vec![1, 2, 3];
//! test(vec.par().copied()); // use a ref to vec
//! test(vec.into_par()); // consume vec
//!
//! // other collections can be used similarly
//! let set: HashSet<_> = [1, 2, 3].into_iter().collect();
//! test(set.par().copied());
//! test(set.into_par());
//!
//! let bmap: BTreeMap<_, _> = [('a', 1), ('b', 2), ('c', 3)].into_iter().collect();
//! test(bmap.par().map(|x| x.1).copied());
//! test(bmap.into_par().map(|x| x.1));
//!
//! // any regular/sequential iterator can be parallelized
//! let iter = ["", "a", "bb", "ccc", "dddd"]
//!     .iter()
//!     .skip(1)
//!     .take(3)
//!     .map(|x| x.len());
//! test(iter.par());
//! ```
//!
//! ## Easy to Configure
//!
//! Complexity of distribution of work to parallel threads is boiled down to two straightforward parameters which are easy to reason about:
//! * [`NumThreads`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.NumThreads.html) represents the degree of parallelization. It can be set to one of the two variants:
//!   * `Auto`: All threads will be assumed to be available. This is an upper bound; whenever the computation is not sufficiently challenging, this number may not be reached.
//!   * `Max(n)`: The computation can spawn at most n threads. NumThreads::Max(1) is equivalent to sequential execution.
//! * [`ChunkSize`](https://docs.rs/orx-parallel/latest/orx_parallel/struct.ChunkSize.html) represents the number of elements a parallel worker will pull and process every time it becomes idle. This parameter aims to balance the overhead of parallelization and cost of heterogeneity of tasks. It can be set to one of the three variants:
//!   * `Auto`: The library aims to select the best value in order to minimize computation time.
//!   * `Exact(c)`: Chunk sizes will be c. This variant gives the control completely to the caller, and hence, suits best to computations to be tuned.
//!   * `Min(c)`: Chunk sizes will be at least c. However, the execution is allowed to pull more elements depending on characteristics of the inputs and used number of threads in order to reduce the impact of parallelization overhead.
//!
//! ```rust
//! use orx_parallel::*;
//! use std::num::NonZeroUsize;
//!
//! let _ = (0..42).par().sum(); // both settings at Auto
//!
//! let _ = (0..42).par().num_threads(4).sum(); // at most 4 threads
//! let _ = (0..42).par().num_threads(1).sum(); // sequential
//! let _ = (0..42).par().num_threads(NumThreads::sequential()).sum(); // also sequential
//! let _ = (0..42).par().num_threads(0).sum(); // shorthand for NumThreads::Auto
//!
//! let _ = (0..42).par().chunk_size(16).sum(); // chunks of exactly 16 elements
//! let c = NonZeroUsize::new(64).unwrap();
//! let _ = (0..42).par().chunk_size(ChunkSize::Min(c)).sum(); // min 64 elements
//! let _ = (0..42).par().chunk_size(0).sum(); // shorthand for ChunkSize::Auto
//!
//! let _ = (0..42).par().num_threads(4).chunk_size(16).sum(); // set both params
//! ```
//!
//! As the example demonstrates, each computation can be configured independently.
//!
//! **|> Better Strategy by Problem Knowledge**
//!
//! Both number of threads and chunk size have `Auto` settings which perform well in general. However, there exists no strategy that performs best for all computations or input characteristics. Sometimes we know the best configuration.
//!
//! For instance, consider a problem where we want to `find` the first feasible or acceptable solution, where each trial is computationally expensive. Of course, we want the computation to terminate as soon as possible once a solution is found.
//! * Setting chunk size to 1 allows for the quickest termination once the solution is found. However, it would have the greatest parallelization overhead.
//! * Setting a sufficiently large chunk size would have much lower parallelization overhead. However, once a thread finds a solution, the other threads would still need to complete their chunk before termination.
//!
//! In this scenario where the computation is challenging, parallelization overhead is negligible. Therefore, we could simply set the chunk size to one.  A similar example is constructed in [benches/map_find_expensive.rs](https://github.com/orxfun/orx-parallel/blob/main/benches/map_find_expensive.rs), where this trivial strategy indeed turns out to be optimal.
//! * `Auto` chunk size settings, as well as, `rayon`'s default settings find the solution in slightly longer time than the sequential computation. This is an example case where parallelization can backfire.
//! * On the other hand, simply setting the chunk size to 1, we observe that `Par` finds the solution ~15 times faster than the sequential.
//!
//! **|> Better Strategy by Tuning**
//!
//! A well known concern in parallelization is the diminishing returns from added resources. Doubling the number of threads usually does not halve the computation time. We usually seek a *sweet spot* where the gain is justified by the allocated resources. Further, same computation might perform significantly differently for different input sets.
//!
//! Being able to have complete control on these parameters allows to benchmark and tune performance-critical computations for the relevant inputs on target platforms.
//!
//! **|> Parallelization in a Concurrent Application**
//!
//! Consider the following two extreme strategies for an api responding to cpu-heavy computation requests:
//!
//! * **sequential**: We can handle each request sequentially. Since the api will serve multiple requests concurrently, we could still utilize available resources. However, we could be under-utilizing in low-traffic time intervals. Further, we might not be achieving desired average response time per request.
//! * **all-in**: We might allow each request to utilize all available resources when it arrives. Under low-traffic, this could minimize the response time. However, recall the diminishing improvements of additional computing resources allocated to one computation. Although we are keeping resources busy, we could be utilizing them very inefficiently and we could perform worse than than the sequential strategy in high traffic periods.
//!
//! A simple strategy could be to limit maximum number of threads to be used per computation. The limit can be a sweet spot discovered empirically. This could already help in preventing the extreme problems discussed above.
//!
//! A more complicated approach could be to implement a smart resource allocator which dynamically decides on the number of threads of a request depending on the traffic and utilization of resources at the instant the request arrives.
//!
//! ## Generalization of Sequential and Parallel Computation
//!
//! Executing a parallel computation with `NumThreads::Max(1)` is equivalent to a sequential computation, without any parallelization overhead. In this sense, `Par` is a generalization of sequential and parallel computation.
//!
//! In order to illustrate, consider the following function which accepts the definition of a computation as a `Par`. Note that just as sequential iterators, `Par` is lazy. In other words, it is just the definition of the computation. Such a `computation` is passed to the `execute` method together with its settings that can be accessed by `computation.params()`.
//!
//! However, since the method owns the `computation`, it may decide how to execute it. This implementation will go with the given parallel settings. Unless it is Monday, then it will run sequentially.
//!
//! ```rust
//! use orx_parallel::*;
//! use chrono::{Datelike, Local, Weekday};
//! type Output = String;
//!
//! fn execute<C: Par<Item = Output>>(computation: C) -> Vec<Output> {
//!     match Local::now().weekday() {
//!         Weekday::Mon => computation.num_threads(1).collect_vec(),
//!         _ => computation.collect_vec(),
//!     }
//! }
//! ```
//!
//! ## Underlying Approach & Performance
//!
//! This crate has developed as a natural follow up of the [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). You may already find example parallel map, fold and find implementations in the examples. Especially when combined with concurrent collections such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag), implementation of parallel computation has been very straightforward. You may find some details in this [section](https://github.com/orxfun/orx-parallel/blob/main/docs/RelationToRayon.md) and this [discussion](https://github.com/orxfun/orx-parallel/discussions/26).
//!
//! In the [benchmarks](https://github.com/orxfun/orx-parallel/blob/main/benches) defined in this repository, we observe that `Par` is very performant, often on-par with rayon. Using the above-mentioned concurrent collections it also suits well where the results of the computation are collected.
//!
//!
//! ## Relation to rayon
//!
//! See [RelationToRayon](https://github.com/orxfun/orx-parallel/blob/main/docs/RelationToRayon.md) section for a discussion on orx-parallel's similarities and differences from rayon.
//!
//! ## Contributing
//!
//! Contributions are welcome! If you notice an error, have a question or think something could be improved, please open an [issue](https://github.com/orxfun/orx-parallel/issues/new) or create a PR.
//!
//! The goal of v1 is to allow `Par` to cover practical use cases, please open an issue if you have a computation that you cannot express and compute with it.
//!
//! The goal of v2 is to provide a more dynamic and smart parallel executor, please see and join the related discussion [here](https://github.com/orxfun/orx-parallel/discussions/26).
//!
//! ## License
//!
//! This library is licensed under MIT license. See LICENSE for details.

#![warn(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]
#![allow(refining_impl_trait)]

mod chunk_size;
mod core;
mod into;
mod num_threads;
mod par;
mod par_iter;
mod params;

pub use chunk_size::ChunkSize;
pub use into::{as_par::AsPar, into_par::IntoPar, iter_into_par::IterIntoPar};
pub use num_threads::NumThreads;
pub use par::cloned_copied::{ParIntoCloned, ParIntoCopied};
pub use par::collect_into::par_collect_into::ParCollectInto;
pub use par::fallible::Fallible;
pub use par_iter::Par;
pub use params::Params;
