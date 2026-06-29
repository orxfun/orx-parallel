mod env;
mod new_pool;
mod par_thread_pool;
mod pool_impl;

pub use new_pool::Pool;

pub use par_thread_pool::ParThreadPool;
#[cfg(feature = "std")]
pub use pool_impl::BasicPool;
#[cfg(all(feature = "wasm-web-threads", target_arch = "wasm32"))]
pub use pool_impl::WasmWebPool;
#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use pool_impl::init_thread_pool;

#[cfg(all(feature = "std", feature = "wasm-web-threads", target_arch = "wasm32"))]
pub type DefaultPool = pool_impl::WasmWebPool;
#[cfg(all(
    feature = "std",
    not(all(feature = "wasm-web-threads", target_arch = "wasm32"))
))]
pub type DefaultPool = pool_impl::OncePool;
#[cfg(not(feature = "std"))]
pub type DefaultPool = pool_impl::SequentialPool;
