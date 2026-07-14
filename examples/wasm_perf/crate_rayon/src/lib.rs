use rayon::prelude::*;
use serde::Serialize;
use tsp_core::{
    SearchRunOutput, clamp_num_cities, locations as create_locations, run_search_sequential,
    search_candidate,
};
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize)]
struct RunResult {
    best_tour: Vec<usize>,
    best_distance: f64,
    iterations: usize,
    elapsed_ms: f64,
}

#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    wasm_bindgen_rayon::init_thread_pool(num_threads as usize)
}

#[wasm_bindgen]
pub fn locations(num_cities: u32) -> Result<JsValue, JsValue> {
    let locations = create_locations(num_cities);
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

#[wasm_bindgen]
pub fn run_best_tour_par(
    iterations: u32,
    seed: u64,
    _num_threads: u32,
    _chunk_size: u32,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    let iterations = iterations.max(1) as usize;
    let num_cities = clamp_num_cities(num_cities);
    let locations = create_locations(num_cities as u32);

    let started_at = js_sys::Date::now();
    let best = (0..iterations)
        .into_par_iter()
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), &locations))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));
    let elapsed_ms = js_sys::Date::now() - started_at;

    run_output_to_js(SearchRunOutput { best, iterations }, elapsed_ms)
}

#[wasm_bindgen]
pub fn run_best_tour_seq(
    iterations: u32,
    seed: u64,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    let iterations = iterations.max(1) as usize;
    let num_cities = clamp_num_cities(num_cities);
    let locations = create_locations(num_cities as u32);

    let started_at = js_sys::Date::now();
    let output = run_search_sequential(iterations, seed, &locations, start_index);
    let elapsed_ms = js_sys::Date::now() - started_at;
    run_output_to_js(output, elapsed_ms)
}

fn run_output_to_js(output: SearchRunOutput, elapsed_ms: f64) -> Result<JsValue, JsValue> {
    match output.best {
        Some((best_tour, best_distance)) => {
            let result = RunResult {
                best_tour,
                best_distance,
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
