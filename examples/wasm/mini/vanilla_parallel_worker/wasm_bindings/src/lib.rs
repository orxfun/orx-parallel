use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
pub fn compute(input: u32, num_threads: u32) -> u64 {
    computation::compute(input as usize, num_threads as usize)
}

#[wasm_bindgen]
pub fn compute_chunks(input: u32, num_threads: u32, chunk_size: u32) -> u64 {
    computation::compute_chunks(input as usize, num_threads as usize, chunk_size as usize)
}
