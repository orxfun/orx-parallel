mod env;
mod global_pool;
mod new_pool;
mod par_thread_pool;
mod pool_impl;

#[cfg(all(
    feature = "wasm",
    feature = "wasm_experimental",
    target_arch = "wasm32"
))]
compile_error!(
    "Features 'wasm' and 'wasm_experimental' are mutually exclusive on wasm32; enable only one wasm backend feature."
);

pub use par_thread_pool::ParThreadPool;
#[cfg(feature = "std")]
pub use pool_impl::BasicPool;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use pool_impl::WasmWebPool;
#[cfg(all(feature = "wasm_experimental", target_arch = "wasm32"))]
pub use pool_impl::WasmWebPoolExp;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pool_impl::init_thread_pool;
#[cfg(all(
    feature = "wasm_experimental",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use pool_impl::init_thread_pool;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pool_impl::wasm_web_runtime_info;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use pool_impl::wasm_web_start_worker;

pub use global_pool::{DefaultPool, get_global_pool};
pub use new_pool::Pool;
