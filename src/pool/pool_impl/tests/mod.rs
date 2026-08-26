#[cfg(feature = "std")]
mod basic;

#[cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]
mod wasm_web;
