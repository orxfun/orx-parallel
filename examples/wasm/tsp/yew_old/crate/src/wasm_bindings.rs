use crate::computation::{SearchRunOutput, run_search_parallel, run_search_sequential};
use crate::locations::{clamp_num_cities, create_locations};
use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RunResult {
    pub best_tour: Vec<usize>,
    pub best_distance: f64,
    pub iterations: usize,
    pub elapsed_ms: f64,
}

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
pub fn locations(seed: u64, num_cities: u32) -> Result<JsValue, JsValue> {
    let locations = create_locations(seed, num_cities);
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

#[wasm_bindgen]
pub fn run_best_tour_par(
    iterations: u32,
    seed: u64,
    threads: u32,
    chunk_size: u32,
    num_cities: u32,
) -> Result<JsValue, JsValue> {
    let iterations = iterations.max(1) as usize;
    let threads = threads as usize;
    let chunk_size = chunk_size as usize;
    let num_cities = clamp_num_cities(num_cities);
    let locations = create_locations(seed, num_cities as u32);
    let started_at = Date::now();
    let output = run_search_parallel(iterations, seed, threads, chunk_size, &locations);
    let elapsed_ms = Date::now() - started_at;
    run_output_to_js(output, elapsed_ms)
}

#[wasm_bindgen]
pub fn run_best_tour_seq(iterations: u32, seed: u64, num_cities: u32) -> Result<JsValue, JsValue> {
    let iterations = iterations.max(1) as usize;
    let num_cities = clamp_num_cities(num_cities);
    let locations = create_locations(seed, num_cities as u32);
    let started_at = Date::now();
    let output = run_search_sequential(iterations, seed, &locations);
    let elapsed_ms = Date::now() - started_at;
    run_output_to_js(output, elapsed_ms)
}

fn run_output_to_js(output: SearchRunOutput, elapsed_ms: f64) -> Result<JsValue, JsValue> {
    match output.best {
        Some(solution) => {
            let result = RunResult {
                best_tour: solution.tour,
                best_distance: solution.distance,
                iterations: output.iterations,
                elapsed_ms,
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("failed to serialize result: {e}")))
        }
        None => Err(JsValue::from_str(
            "no tour could be generated (unexpected empty search)",
        )),
    }
}
