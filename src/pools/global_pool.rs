use crate::pools::pool_impl::*;
#[cfg(feature = "std")]
use std::sync::LazyLock;

// PERSISTENT_POOL - Feature precedence:
// 1. wasm (strongest) - on wasm32 target
// 2. persistent-pool-rayon
// 3. transient-pool
// 4. default - BasicPool (if std) or SequentialPool (if no-std)

// 1. wasm on wasm32
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
static PERSISTENT_POOL: LazyLock<WasmWebPool> = LazyLock::new(Default::default);

// 2. persistent-pool-rayon (and not wasm on wasm32)
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(all(feature = "wasm", target_arch = "wasm32")),
))]
static PERSISTENT_POOL: LazyLock<rayon_core::ThreadPool> =
    LazyLock::new(build_default_rayon_thread_pool);

// 3. transient-pool (and not wasm on wasm32, and not persistent-pool-rayon)
#[cfg(all(
    feature = "std",
    feature = "transient-pool",
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
))]
static PERSISTENT_POOL: LazyLock<OncePool> = LazyLock::new(Default::default);

// 4a. default with std (and not wasm on wasm32, and not persistent-pool-rayon, and not transient-pool)
#[cfg(all(
    feature = "std",
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
    not(feature = "transient-pool"),
))]
static PERSISTENT_POOL: LazyLock<BasicPool> = LazyLock::new(Default::default);

// 4b. default without std (fallback - sequential pool)
#[cfg(all(
    not(feature = "std"),
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
    not(feature = "transient-pool"),
))]
static PERSISTENT_POOL: SequentialPool = SequentialPool;

// DEFAULT POOL

// 1. wasm on wasm32
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
/// Default thread pool
pub type DefaultPool = &'static WasmWebPool;

// 2. persistent-pool-rayon (and not wasm on wasm32)
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(all(feature = "wasm", target_arch = "wasm32")),
))]
/// Default thread pool
pub type DefaultPool = &'static rayon_core::ThreadPool;

// 3. transient-pool (and not wasm on wasm32, and not persistent-pool-rayon)
#[cfg(all(
    feature = "std",
    feature = "transient-pool",
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
))]
/// Default thread pool
pub type DefaultPool = &'static OncePool;

// 4a. default with std (and not wasm on wasm32, and not persistent-pool-rayon, and not transient-pool)
#[cfg(all(
    feature = "std",
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
    not(feature = "transient-pool"),
))]
/// Default thread pool
pub type DefaultPool = &'static BasicPool;

// 4b. default without std (fallback - sequential pool)
#[cfg(all(
    not(feature = "std"),
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
    not(feature = "transient-pool"),
))]
/// Default thread pool
pub type DefaultPool = &'static SequentialPool;

// GET POOL

// 1. wasm on wasm32
#[cfg(all(feature = "std", feature = "wasm", target_arch = "wasm32"))]
pub(crate) fn global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 2. persistent-pool-rayon (and not wasm on wasm32)
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(all(feature = "wasm", target_arch = "wasm32")),
))]
pub(crate) fn global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 3. transient-pool (and not wasm on wasm32, and not persistent-pool-rayon)
#[cfg(all(
    feature = "std",
    feature = "transient-pool",
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
))]
pub(crate) fn global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 4a. default with std (and not wasm on wasm32, and not persistent-pool-rayon, and not transient-pool)
#[cfg(all(
    feature = "std",
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
    not(feature = "transient-pool"),
))]
pub(crate) fn global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}

// 4b. default without std (fallback - sequential pool)
#[cfg(all(
    not(feature = "std"),
    not(all(feature = "wasm", target_arch = "wasm32")),
    not(feature = "persistent-pool-rayon"),
    not(feature = "transient-pool"),
))]
/// Returns the default global thread pool.
///
/// This is handy for **ad-hoc** parallel work: spawn a few tasks into a [`scope`]
/// and let them run side by side. Note, however, that this pool plays only a
/// minimal role in this crate's performance; the real speedups come from the
/// concurrent iterators and parallel runner strategies used underneath parallel
/// iterators. So prefer expressing computations through parallel iterators, and
/// avoid calling `s.run` thousands of times in a loop, as this pool is not
/// optimized for that.
///
/// # Examples
///
/// ```
/// use orx_parallel::{Scope, ThreadPool, global_pool};
///
/// global_pool().scope(|s| {
///     s.run(|| println!("computing part A"));
///     s.run(|| println!("computing part B"));
/// });
/// ```
///
/// [`scope`]: crate::ThreadPool::scope
pub(crate) fn global_pool() -> DefaultPool {
    &PERSISTENT_POOL
}
