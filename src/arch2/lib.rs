//! # orx-parallel
//!
//! [![orx-parallel crate](https://img.shields.io/crates/v/orx-parallel.svg)](https://crates.io/crates/orx-parallel)
//! [![orx-parallel documentation](https://docs.rs/orx-parallel/badge.svg)](https://docs.rs/orx-parallel)
//!
//! A performant and configurable parallel computing library for computations defined as compositions of iterator methods.
//!
//! ## Parallel Computation by Iterators
//!
//! Parallel computation is achieved conveniently by the parallel iterator trait [`Par`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.Par.html). This allows for changing sequential code that is defined as a composition of functions through iterators into its parallel counterpart by adding one word: `par` or `into_par`.
//!
//! ```rust
//! use orx_parallel::prelude::*;
//!
//! struct Input(String);
//! struct Output(usize);
//!
//! let compute = |input: Input| Output(input.0.len());
//! let select = |output: &Output| output.0.is_power_of_two();
//!
//! let inputs = || (0..1024).map(|x| Input(x.to_string())).collect::<Vec<_>>();
//!
//! // sequential computation with regular iterator
//! let seq_result: usize = inputs()
//!     .into_iter()
//!     .map(compute)
//!     .filter(select)
//!     .map(|x| x.0)
//!     .sum();
//! assert_eq!(seq_result, 286);
//!
//! // parallel computation with Par
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
//! use orx_parallel::prelude::*;
//! use std::collections::*;
//!
//! fn test<P: Par<Item = usize>>(iter: P) {
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
//! use orx_parallel::prelude::*;
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
//! Having control on these two parameters and being able to configure each computation easily and individually is useful in various ways. See
//! [EasyConfiguration](https://github.com/orxfun/orx-parallel/blob/main/docs/EasyConfiguration.md) section for examples.
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
//! use orx_parallel::prelude::*;
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
//! This features saves us from defining the same computation twice. We are often required to write code like below where we need to run sequentially or in parallel depending on an input argument. This is repetitive, error-prone and difficult to maintain.
//!
//! ```rust
//! use orx_parallel::prelude::*;
//! struct Input(String);
//! struct Output(usize);
//! fn compute(input: Input) -> Output {
//!     Output(input.0.len())
//! }
//! fn select(output: &Output) -> bool {
//!     output.0.is_power_of_two()
//! }
//!
//! fn execute_conditionally(inputs: impl Iterator<Item = Input>, parallelize: bool) -> usize {
//!     match parallelize {
//!         true => inputs
//!             .into_iter()
//!             .par()
//!             .map(compute)
//!             .filter(select)
//!             .map(|x| x.0)
//!             .sum(),
//!         false => inputs
//!             .into_iter()
//!             .map(compute)
//!             .filter(select)
//!             .map(|x| x.0)
//!             .sum(),
//!     }
//! }
//! ```
//!
//! Using `Par`, we can have a single version which will not have any overhead when executed sequentially.
//!
//! ```rust
//! # use orx_parallel::prelude::*;
//! # struct Input(String);
//! # struct Output(usize);
//! # fn compute(input: Input) -> Output {
//! #     Output(input.0.len())
//! # }
//! # fn select(output: &Output) -> bool {
//! #     output.0.is_power_of_two()
//! # }
//! fn execute_unified(inputs: impl Iterator<Item = Input>, parallelize: bool) -> usize {
//!     let num_threads = match parallelize {
//!         true => NumThreads::Auto,
//!         false => NumThreads::sequential(),
//!     };
//!     inputs
//!         .par()
//!         .num_threads(num_threads)
//!         .map(compute)
//!         .filter(select)
//!         .map(|x| x.0)
//!         .sum()
//! }
//! ```
//!
//! ## Underlying Approach & Performance
//!
//! This crate has developed as a natural follow up of the [`ConcurrentIter`](https://crates.io/crates/orx-concurrent-iter). You may already find example parallel map, fold and find implementations in the examples. Especially when combined with concurrent collections such as [`ConcurrentBag`](https://crates.io/crates/orx-concurrent-bag) and [`ConcurrentOrderedBag`](https://crates.io/crates/orx-concurrent-ordered-bag), implementation of parallel computation has been very straightforward. You may find some details in this [section](https://github.com/orxfun/orx-parallel/blob/main/docs/RelationToRayon.md) and this [discussion](https://github.com/orxfun/orx-parallel/discussions/26).
//!
//! Benchmarks are tricky, even more in parallel context. Nevertheless, results of [benchmarks](https://github.com/orxfun/orx-parallel/blob/main/benches) defined in this repository are very promising for `Par`. Its performance is often on-par with rayon. It can provide significant improvements in scenarios where the results are collected, such as [map |> filter |> collect](https://github.com/orxfun/orx-parallel/blob/main/benches/map_filter_collect.rs) or [flat_map |> collect](https://github.com/orxfun/orx-parallel/blob/main/benches/flatmap.rs), etc.
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
    // missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]
// #![allow(refining_impl_trait)]

mod collect_into;
mod computations;
mod into_par;
mod par_iterators;
mod parallelizable;
mod parallelizable_collection;
mod parameters;

pub use collect_into::{ParCollectInto, ParCollectIntoCore};
pub use into_par::{IntoPar, IteratorIntoPar};
pub use par_iterators::ParIter;
pub use parallelizable::Parallelizable;
pub use parallelizable_collection::ParallelizableCollection;
