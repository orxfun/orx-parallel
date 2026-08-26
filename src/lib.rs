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
#![no_std]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod collectables;
mod common_par_traits;
/// Core module for infallible computations.
pub mod infallible;
mod infallible_use;
mod into_parallel;
mod ops;
mod option;
mod option_use;
mod parameters;
/// Thread pool module
pub mod pool;
mod result;
mod result_use;
mod results;
mod runner;
mod sizes;
mod use_var;

pub use collectables::{ParCollectInto, Vec2};
pub use infallible::{EnumeratePar, Par, ParRec};
pub use infallible_use::{EnumerateParUse, ParUse};
pub use into_parallel::{
    IntoParIter, IntoParRec, IterIntoParIter, ParCollection, ParCollectionMut, ParDrain,
    Parallelizable,
};
pub use ops::{ParExtend, Sum};
pub use option::{ParOption, ParRecOption};
pub use option_use::ParUseOption;
pub use parameters::{ChunkSize, IterationOrder, NumThreads, Params};
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use pool::WasmWebPool;

#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pool::wasm_web_runtime_info;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pool::wasm_web_start_worker;
pub use pool::{ParThreadPool, Pool};
pub use result::{ParRecResult, ParResult};
pub use result_use::ParUseResult;
pub use runner::Runner;
pub use use_var::{Use, UseVec};

/// Initializes the browser's shared wasm thread pool.
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pool::init_wasm_parallel_runtime;
