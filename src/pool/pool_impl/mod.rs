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

#[cfg(feature = "std")]
mod basic;
#[cfg(feature = "std")]
pub use basic::BasicPool;
