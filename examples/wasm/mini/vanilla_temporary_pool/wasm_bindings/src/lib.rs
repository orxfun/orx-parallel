use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// Initializes the shared thread pool used by the parallel computation.
///
/// This function must be called once before invoking `compute`.
pub fn init_wasm_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_wasm_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!(
        "init_wasm_parallel_runtime requires a wasm target with atomics and shared memory enabled"
    )
}

#[wasm_bindgen]
pub fn compute(input: u32, num_threads: u32) -> u64 {
    computation::compute(input as usize, num_threads as usize)
}
