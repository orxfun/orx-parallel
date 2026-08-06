use crate::pool::pool_impl::*;
#[cfg(feature = "std")]
use std::sync::LazyLock;

// PERSISTENT_POOL

#[cfg(all(feature = "std", feature = "wasm_experimental", target_arch = "wasm32"))]
static PERSISTENT_POOL: LazyLock<WasmWebPoolExp> = LazyLock::new(Default::default);

#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    not(feature = "wasm_experimental"),
))]
static PERSISTENT_POOL: LazyLock<WasmWebPool> = LazyLock::new(Default::default);

#[cfg(all(
    feature = "std",
    feature = "persistent_pool_rayon",
    not(feature = "wasm"),
    not(feature = "wasm_experimental"),
))]
static PERSISTENT_POOL: LazyLock<rayon_core::ThreadPool> =
    LazyLock::new(build_default_rayon_thread_pool);

#[cfg(all(
    feature = "std",
    feature = "persistent_pool",
    not(feature = "persistent_pool_rayon"),
    not(feature = "wasm"),
    not(feature = "wasm_experimental"),
))]
static PERSISTENT_POOL: LazyLock<BasicPool> = LazyLock::new(Default::default);

// DefaultPool

#[cfg(all(feature = "std", feature = "wasm_experimental", target_arch = "wasm32"))]
pub type DefaultPool = WasmWebPoolExp;

#[cfg(all(
    feature = "std",
    feature = "wasm",
    target_arch = "wasm32",
    not(feature = "wasm_experimental"),
))]
pub type DefaultPool = WasmWebPool;

#[cfg(all(
    feature = "std",
    feature = "persistent_pool_rayon",
    not(feature = "wasm"),
    not(feature = "wasm_experimental"),
))]
pub type DefaultPool = rayon_core::ThreadPool;

#[cfg(all(
    feature = "std",
    feature = "persistent_pool",
    not(feature = "persistent_pool_rayon"),
    not(feature = "wasm"),
    not(feature = "wasm_experimental"),
))]
pub type DefaultPool = BasicPool;

#[cfg(all(
    feature = "std",
    not(feature = "persistent_pool"),
    not(feature = "persistent_pool_rayon"),
    not(feature = "wasm"),
    not(feature = "wasm_experimental"),
))]
pub type DefaultPool = OncePool;

#[cfg(all(
    not(feature = "std"),
    not(feature = "persistent_pool"),
    not(feature = "persistent_pool_rayon"),
    not(feature = "wasm"),
    not(feature = "wasm_experimental"),
))]
pub type DefaultPool = SequentialPool;

// access
