#![doc = include_str!("../README.md")]
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
// #![no_std]

extern crate alloc;

// #[cfg(any(test, feature = "std"))]
extern crate std;

mod collect_into;
/// Module containing variants of parallel iterators.
pub mod computational_variants;
mod default_fns;
mod env;
/// Module defining the parallel runner trait and the default parallel runner.
pub mod executor;
mod generic_values;
mod heap_sort;
mod into_par_iter;
/// Module for creating special iterators.
pub mod iter;
mod iter_into_par_iter;
mod par_iter;
mod par_iter_option;
mod par_iter_result;
mod par_thread_pool;
mod parallel_drainable;
mod parallelizable;
mod parallelizable_collection;
mod parallelizable_collection_mut;
mod parameters;
/// ParallelRunner for parallel execution and managing threads.
pub mod runner;
mod special_type_sets;
/// Module defining parallel iterators with mutable access to values distributed to each thread.
pub mod using;

/// Module defining the GenericIterator which is a generalization over
/// sequential iterator, rayon's parallel iterator and orx-parallel's
/// parallel iterator.
/// This is particularly useful for running experiments and comparing
/// results of computations with different methods.
#[cfg(feature = "generic_iterator")]
pub mod generic_iterator;

/// Module with utility methods for testing.
#[cfg(test)]
mod test_utils;

pub use collect_into::ParCollectInto;
pub use executor::{
    DefaultExecutor, ParallelExecutor, ParallelExecutorWithDiagnostics, ThreadExecutor,
};
pub use into_par_iter::IntoParIter;
// pub use iter::{IntoParIterRec, IntoParIterRecExact};
pub use iter_into_par_iter::IterIntoParIter;
pub use par_iter::ParIter;
pub use par_iter_option::ParIterOption;
pub use par_iter_result::ParIterResult;
pub use par_thread_pool::ParThreadPool;
pub use parallel_drainable::ParallelDrainableOverSlice;
pub use parallelizable::Parallelizable;
pub use parallelizable_collection::ParallelizableCollection;
pub use parallelizable_collection_mut::ParallelizableCollectionMut;
pub use parameters::{ChunkSize, IterationOrder, NumThreads, Params};
pub use special_type_sets::Sum;
pub use using::ParIterUsing;

pub use runner::{DefaultPool, DefaultRunner, ParallelRunner, RunnerWithPool, SequentialPool};

#[cfg(feature = "pond")]
pub use runner::PondPool;
#[cfg(feature = "std")]
pub use runner::StdDefaultPool;
#[cfg(feature = "yastl")]
pub use runner::YastlPool;
