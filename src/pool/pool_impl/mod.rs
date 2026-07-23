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

#[cfg(all(feature = "wasm-web-threads-experimental", target_arch = "wasm32"))]
mod wasm_web_exp;
#[cfg(all(feature = "wasm-web-threads-experimental", target_arch = "wasm32"))]
pub use wasm_web_exp::WasmWebPoolExp;
#[cfg(all(
    feature = "wasm-web-threads-experimental",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use wasm_web_exp::init_thread_pool;

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
#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use wasm_web::wasm_web_runtime_info;
#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use wasm_web::wasm_web_start_worker;

#[cfg(feature = "std")]
mod basic;
#[cfg(feature = "std")]
pub use basic::BasicPool;
