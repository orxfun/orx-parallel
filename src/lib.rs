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
#![no_std]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod collectables;
mod infallible;
mod infallible_use;
mod into_parallel;
mod kind_transformations;
mod option;
mod option_use;
mod parameters;
mod pool;
mod result;
mod result_use;
mod results;
mod runner;
mod sizes;

pub use collectables::ParCollectInto;
pub use infallible::ParIter;
pub use infallible_use::ParUseIter;
pub use into_parallel::{
    IntoParIter, IntoParIterRecursive, IterIntoParIter, ParCol, ParColMut, ParDrain, Parallelizable,
};
pub use option::ParOptIter;
pub use option_use::ParUseOptIter;
pub use parameters::{ChunkSize, IterationOrder, NumThreads, Params};
pub use result::ParResIter;
