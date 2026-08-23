use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compute(input: u32, num_threads: u32) -> u64 {
    computation::compute(input as usize, num_threads as usize)
}

#[wasm_bindgen]
pub fn compute_chunks(input: u32, num_threads: u32, chunk_size: u32) -> u64 {
    computation::compute_chunks(input as usize, num_threads as usize, chunk_size as usize)
}
