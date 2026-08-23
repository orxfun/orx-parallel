use crate::pool::pool_impl::*;
#[cfg(feature = "std")]
use std::sync::LazyLock;

// PERSISTENT_POOL

// 0. all-features on native targets
#[cfg(all(
    feature = "std",
    feature = "wasm",
    feature = "persistent-pool",
    feature = "persistent-pool-rayon",
    not(target_arch = "wasm32")
))]
static PERSISTENT_POOL: LazyLock<OncePool> = LazyLock::new(Default::default);

// 0. wasm
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
static PERSISTENT_POOL: LazyLock<WasmWebPool> = LazyLock::new(Default::default);

// 1. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
))]
static PERSISTENT_POOL: LazyLock<rayon_core::ThreadPool> =
    LazyLock::new(build_default_rayon_thread_pool);

// 2. basic
#[cfg(all(
    feature = "std",
    feature = "persistent-pool",
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
static PERSISTENT_POOL: LazyLock<BasicPool> = LazyLock::new(Default::default);

// 3. once
#[cfg(all(
    feature = "std",
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
static PERSISTENT_POOL: LazyLock<OncePool> = LazyLock::new(Default::default);

// DEFAULT POOL

// 0. all-features on native targets
#[cfg(all(
    feature = "std",
    feature = "wasm",
    feature = "persistent-pool",
    feature = "persistent-pool-rayon",
    not(target_arch = "wasm32")
))]
/// Default thread pool
pub type DefaultPool = &'static OncePool;

// 0. wasm
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
/// Default thread pool
pub type DefaultPool = &'static WasmWebPool;

// 1. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
))]
/// Default thread pool
pub type DefaultPool = &'static rayon_core::ThreadPool;

// 2. basic
#[cfg(all(
    feature = "std",
    feature = "persistent-pool",
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
/// Default thread pool
pub type DefaultPool = &'static BasicPool;

// 3. once
#[cfg(all(
    feature = "std",
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
/// Default thread pool
pub type DefaultPool = &'static OncePool;

// 4. sequential
#[cfg(all(
    not(feature = "std"),
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
/// Default thread pool
pub type DefaultPool = SequentialPool;

// GET POOL

// 0. all-features on native targets
#[cfg(all(
    feature = "std",
    feature = "wasm",
    feature = "persistent-pool",
    feature = "persistent-pool-rayon",
    not(target_arch = "wasm32")
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 0. wasm
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 1. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 2. basic
#[cfg(all(
    feature = "std",
    feature = "persistent-pool",
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 3. once
#[cfg(all(
    feature = "std",
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 4. sequential
#[cfg(all(
    not(feature = "std"),
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    Default::default()
}
