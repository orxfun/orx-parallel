#![cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]

#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
use wasm_bindgen::prelude::*;

#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
pub use orx_parallel::*;

#[cfg(all(
    feature = "wasm-web-threads",
    target_arch = "wasm32",
    target_feature = "atomics"
))]
#[wasm_bindgen]
pub fn parallel_sum(n: u32) -> u64 {
    let pool = Pool::wasm_web(4);

    (0..n as usize)
        .into_par()
        .pool(pool)
        .map(|x| x as u64)
        .reduce(|a, b| a + b)
        .unwrap_or(0)
}

fn main() {}
