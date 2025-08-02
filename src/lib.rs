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

mod collect_into;
/// Module containing variants of parallel iterators.
pub mod computational_variants;
mod computations;
mod into_par_iter;
/// Module for creating special iterators.
pub mod iter;
mod iter_into_par_iter;
mod par_iter;
mod parallel_drainable;
mod parallelizable;
mod parallelizable_collection;
mod parameters;
/// Module defining the parallel runner trait and the default parallel runner.
pub mod runner;
mod special_type_sets;
mod u_par_iter;

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
pub use into_par_iter::IntoParIter;
pub use iter_into_par_iter::IterIntoParIter;
pub use par_iter::ParIter;
pub use parallel_drainable::ParallelDrainableOverSlice;
pub use parallelizable::Parallelizable;
pub use parallelizable_collection::ParallelizableCollection;
pub use parameters::{ChunkSize, IterationOrder, NumThreads, Params};
pub use runner::{DefaultRunner, ParallelRunner, ThreadRunner};
pub use special_type_sets::Sum;
