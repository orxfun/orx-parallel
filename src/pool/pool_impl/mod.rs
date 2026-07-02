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

#[cfg(all(feature = "wasm-web-threads", target_arch = "wasm32"))]
mod wasm_web;
#[cfg(all(feature = "wasm-web-threads", target_arch = "wasm32"))]
pub use wasm_web::WasmWebPool;
#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use wasm_web::init_thread_pool;

#[cfg(all(feature = "wasm-web-threads2", target_arch = "wasm32"))]
mod wasm_web2;
#[cfg(all(feature = "wasm-web-threads2", target_arch = "wasm32"))]
pub use wasm_web2::WasmWebPool2;
#[cfg(all(
    feature = "wasm-web-threads2",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use wasm_web2::init_thread_pool;

#[cfg(feature = "std")]
mod basic;
#[cfg(feature = "std")]
pub use basic::BasicPool;
