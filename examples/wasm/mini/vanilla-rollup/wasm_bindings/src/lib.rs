use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn calculate_fibonacci(workload: u32, num_threads: u32) -> u64 {
    computation::calculate_fibonacci(workload as usize, num_threads as usize)
}

#[wasm_bindgen]
pub fn mandelbrot_checksum(limit: u32, num_threads: u32) -> u32 {
    computation::mandelbrot_checksum(limit as usize, num_threads as usize) as u32
}
