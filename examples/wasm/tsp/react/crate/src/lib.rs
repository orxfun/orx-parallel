mod computation;
mod locations;

#[cfg(target_feature = "atomics")]
mod wasm_bindings;

pub use computation::{run_search_parallel, run_search_sequential};
pub use locations::create_locations;
