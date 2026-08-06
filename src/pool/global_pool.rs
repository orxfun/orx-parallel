use crate::pool::pool_impl::*;
#[cfg(feature = "std")]
use std::sync::LazyLock;

// PERSISTENT_POOL

// 1. wasm-experimental
#[cfg(all(feature = "std", feature = "wasm-experimental", target_arch = "wasm32"))]
static PERSISTENT_POOL: LazyLock<WasmWebPoolExp> = LazyLock::new(Default::default);

// 2. wasm
#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    not(feature = "wasm-experimental"),
))]
static PERSISTENT_POOL: LazyLock<WasmWebPool> = LazyLock::new(Default::default);

// 3. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
static PERSISTENT_POOL: LazyLock<rayon_core::ThreadPool> =
    LazyLock::new(build_default_rayon_thread_pool);

// 4. basic
#[cfg(all(
    feature = "std",
    feature = "persistent-pool",
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
static PERSISTENT_POOL: LazyLock<BasicPool> = LazyLock::new(Default::default);

// 5. once
#[cfg(all(
    feature = "std",
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
static PERSISTENT_POOL: LazyLock<OncePool> = LazyLock::new(Default::default);

// DEFAULT POOL

// 1. wasm-experimental
#[cfg(all(feature = "std", feature = "wasm-experimental", target_arch = "wasm32"))]
/// Default thread pool
pub type DefaultPool = &'static WasmWebPoolExp;

// 2. wasm
#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    not(feature = "wasm-experimental"),
))]
/// Default thread pool
pub type DefaultPool = &'static WasmWebPool;

// 3. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Default thread pool
pub type DefaultPool = &'static rayon_core::ThreadPool;

// 4. basic
#[cfg(all(
    feature = "std",
    feature = "persistent-pool",
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Default thread pool
pub type DefaultPool = &'static BasicPool;

// 5. once
#[cfg(all(
    feature = "std",
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Default thread pool
pub type DefaultPool = &'static OncePool;

// 6. sequential
#[cfg(all(
    not(feature = "std"),
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Default thread pool
pub type DefaultPool = SequentialPool;

// GET POOL

// 1. wasm-experimental
#[cfg(all(feature = "std", feature = "wasm-experimental", target_arch = "wasm32"))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 2. wasm
#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    not(feature = "wasm-experimental"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 3. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 4. basic
#[cfg(all(
    feature = "std",
    feature = "persistent-pool",
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 5. once
#[cfg(all(
    feature = "std",
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 6. sequential
#[cfg(all(
    not(feature = "std"),
    not(feature = "persistent-pool"),
    not(feature = "persistent-pool-rayon"),
    not(feature = "wasm"),
    not(feature = "wasm-experimental"),
))]
/// Returns the default global thread pool.
pub fn get_global_pool() -> DefaultPool {
    Default::default()
}
