use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
pub fn calculate_fibonacci(workload: u32, num_threads: u32) -> u64 {
    computation::calculate_fibonacci(workload as usize, num_threads as usize)
}

#[wasm_bindgen]
pub fn count_primes(limit: u32, num_threads: u32) -> u32 {
    computation::count_primes(limit as usize, num_threads as usize) as u32
}
