//! # orx-parallel

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
mod into_par;
mod num_threads;
mod par;
mod par_iter;
mod params;

pub use chunk_size::ChunkSize;
pub use into_par::{IntoPar, IterPar};
pub use num_threads::NumThreads;
pub use par::collect_into::par_collect_into::ParCollectInto;
pub use par::fallible::Fallible;
pub use par::{
    par_empty::Par, par_fil::ParFilter, par_map::ParMap, par_map_fil::ParMapFilter, reduce::Reduce,
};
pub use par_iter::ParIter;
pub use params::Params;
