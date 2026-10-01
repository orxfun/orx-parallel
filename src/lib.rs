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

/// Extendable parallel collection helpers.
pub mod extendable;
/// Core module for infallible computations.
pub mod infallible;
mod infallible_use;
mod into_parallel;
mod ops;
mod option;
mod option_use;
mod parameters;
/// Thread pools.
pub mod pools;
mod result;
mod result_use;
mod results;
mod runner;
mod sizes;
mod sort;
mod use_var;

pub use extendable::ParExtend;
pub use infallible::{EnumeratePar, Par, ParRec};
pub use infallible_use::{EnumerateParUse, ParUse};
pub use into_parallel::{
    IntoParIter, IterIntoParIter, ParCollection, ParCollectionMut, ParDrain, Parallelizable,
    par_recursive,
};
pub use ops::Sum;
pub use option::ParOption;
pub use option_use::ParUseOption;
pub use parameters::{ChunkSize, IterationOrder, NumThreads, Params};
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use pools::WasmWebPool;

/// Re-export the optional sharded allocator for atomics-enabled WebAssembly.
#[cfg(all(feature = "wasm-allocator", target_arch = "wasm32"))]
pub use orx_parallel_wasm_allocator::WasmParallelAllocator;

#[cfg(not(feature = "std"))]
pub use pools::SequentialPool;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pools::wasm_web_runtime_info;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pools::wasm_web_start_worker;
#[cfg(feature = "std")]
pub use pools::{BasicPool, OncePool};
pub use pools::{Pool, Scope, TaskQueue, Tasks, ThreadPool};
pub use result::ParResult;
pub use result_use::ParUseResult;
pub use runner::{ParRunner, Runner};
pub use use_var::{Use, UseVec};

/// Initializes the browser's shared wasm thread pool.
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pools::init_wasm_parallel_runtime;

// experimental
#[cfg(feature = "experimental")]
pub use sort::par_experimental_sort;
