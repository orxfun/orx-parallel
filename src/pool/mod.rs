mod env;
mod global_pool;
mod new_pool;
mod par_thread_pool;
mod pool_impl;

// 1. wasm
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
pub use pool_impl::WasmWebPool;

#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    target_feature = "atomics",
))]
pub use pool_impl::init_thread_pool;

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
#[cfg(all(
    not(feature = "std"),
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
pub use pool_impl::SequentialPool;

pub use global_pool::{DefaultPool, get_global_pool};
pub use new_pool::Pool;
pub use par_thread_pool::ParThreadPool;
