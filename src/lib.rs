#![doc = include_str!("../README.md")]
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

mod collect_into;
pub mod computational_variants;
mod computations;
mod into_par_iter;
/// Module for creating special iterators.
pub mod iter;
mod iter_into_par_iter;
mod par_iter;
mod parallelizable;
mod parallelizable_collection;
mod parameters;
pub mod runner;
mod special_type_sets;

#[cfg(feature = "generic_iterator")]
pub mod generic_iterator;

#[cfg(test)]
mod test_utils;

pub use collect_into::ParCollectInto;
pub use into_par_iter::IntoParIter;
pub use iter_into_par_iter::IterIntoParIter;
pub use par_iter::ParIter;
pub use parallelizable::Parallelizable;
pub use parallelizable_collection::ParallelizableCollection;
pub use parameters::{ChunkSize, CollectOrdering, NumThreads, Params};
pub use runner::{DefaultRunner, ParallelRunner, ThreadRunner};
pub use special_type_sets::Sum;
