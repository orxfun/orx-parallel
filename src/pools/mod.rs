mod env;
mod global_pool;
mod pool;
mod pool_impl;
mod scope;
pub mod tasks;
mod thread_pool;

// 1. wasm
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
pub use pool_impl::WasmWebPool;

#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    target_feature = "atomics",
))]
pub use pool_impl::init_wasm_parallel_runtime;

#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    target_feature = "atomics",
))]
pub use pool_impl::wasm_web_runtime_info;

#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    target_feature = "atomics",
))]
pub use pool_impl::wasm_web_start_worker;

// 2. basic
#[cfg(feature = "std")]
pub use pool_impl::BasicPool;

// 3. once
#[cfg(feature = "std")]
pub use pool_impl::OncePool;

// 4. sequential
#[cfg(not(feature = "std"))]
pub use pool_impl::SequentialPool;

pub use global_pool::DefaultPool;
pub(crate) use global_pool::global_pool;
pub use pool::Pool;
pub use scope::Scope;
pub use tasks::{TaskQueue, Tasks};
pub use thread_pool::{ThreadPool, max_num_threads_for_computation};
