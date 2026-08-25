#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
mod once;

#[cfg(feature = "std")]
pub use once::OncePool;

#[cfg(not(feature = "std"))]
mod sequential;
#[cfg(not(feature = "std"))]
pub use sequential::SequentialPool;

#[cfg(feature = "rayon-core")]
mod rayon_core;
// 3. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(feature = "wasm"),
))]
pub use rayon_core::build_default_rayon_thread_pool;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm_web;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use wasm_web::WasmWebPool;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use wasm_web::init_wasm_parallel_runtime;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub(super) use wasm_web::init_wasm_thread_pool;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use wasm_web::wasm_web_runtime_info;
#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
pub use wasm_web::wasm_web_start_worker;

#[cfg(feature = "std")]
mod basic;
#[cfg(feature = "std")]
pub use basic::BasicPool;
