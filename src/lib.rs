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
mod parallelizable;
mod parallelizable_collection;
mod parameters;
/// Module defining the parallel runner trait and the default parallel runner.
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn abc() {
        let data: Vec<_> = (0..100).map(|x| x.to_string()).collect();

        // an arbitrary iterator
        let iter = data
            .into_iter()
            .filter(|x| !x.starts_with('3'))
            .map(|x| format!("{x}!"));

        // convert arbitrary iterator into ParIter
        let par_iter = iter.iter_into_par();
        let num_characters = par_iter.map(|x| x.len()).sum();

        assert_eq!(num_characters, 258);
    }
}
