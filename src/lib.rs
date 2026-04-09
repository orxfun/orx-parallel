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

mod into_parallel;
mod parameters;
mod pool;
mod results;
mod runner;

pub mod infallible; // TODO: pub only for benchmarks, can we prevent this?
mod infallible_use;
mod kind_transformations;
mod option;
mod option_use;
mod result;
mod result_use;
mod sizes;

pub use into_parallel::{
    IntoParIter, IterIntoParIter, ParCol, ParColMut, ParDrain, Parallelizable,
};
