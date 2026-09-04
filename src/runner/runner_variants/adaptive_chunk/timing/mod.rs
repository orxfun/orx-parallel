#[cfg(not(target_arch = "wasm32"))]
mod timing_non_wasm;
#[cfg(target_arch = "wasm32")]
mod timing_wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use timing_non_wasm::{Instant, Timing};
#[cfg(target_arch = "wasm32")]
pub use timing_wasm::{Instant, Timing};
