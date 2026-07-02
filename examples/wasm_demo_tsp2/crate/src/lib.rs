use serde::Serialize;
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
mod computation;
mod locations;

#[cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]
#[derive(Debug, Serialize)]
struct RunResult {
    best_tour: Vec<usize>,
    best_distance: f64,
    iterations: usize,
    elapsed_ms: f64,
}

/// Initializes the wasm worker thread pool used by parallel runs.
#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    orx_parallel::init_thread_pool(num_threads as usize)
}

/// Returns an error when thread-pool initialization is unavailable on this target.
#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn init_parallel_runtime(_num_threads: u32) -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "init_parallel_runtime is only available for wasm32 + atomics builds",
    ))
}

/// Returns the city coordinates for the requested problem size.
#[wasm_bindgen]
pub fn locations(num_cities: u32) -> Result<JsValue, JsValue> {
    let locations = locations::locations(num_cities);
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

/// Runs a parallel TSP search chunk and returns the best tour found in that chunk.
#[wasm_bindgen]
pub fn run_best_tour_par(
    iterations: u32,
    seed: u64,
    threads: u32,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, threads, num_cities, start_index);
        return Err(JsValue::from_str(
            "run_best_tour_par requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let threads = threads.max(1) as usize;
        let num_cities = locations::clamp_num_cities(num_cities);
        let locations = locations::locations(num_cities as u32);
        let started_at = js_sys::Date::now();
        let output =
            computation::run_search_parallel(iterations, seed, threads, &locations, start_index);
        let elapsed_ms = js_sys::Date::now() - started_at;
        run_output_to_js(output, elapsed_ms)
    }
}

/// Runs a sequential TSP search chunk and returns the best tour found in that chunk.
#[wasm_bindgen]
pub fn run_best_tour_seq(
    iterations: u32,
    seed: u64,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, num_cities, start_index);
        return Err(JsValue::from_str(
            "run_best_tour_seq requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let num_cities = locations::clamp_num_cities(num_cities);
        let locations = locations::locations(num_cities as u32);
        let started_at = js_sys::Date::now();
        let output = computation::run_search_sequential(iterations, seed, &locations, start_index);
        let elapsed_ms = js_sys::Date::now() - started_at;
        run_output_to_js(output, elapsed_ms)
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn run_output_to_js(
    output: computation::SearchRunOutput,
    elapsed_ms: f64,
) -> Result<JsValue, JsValue> {
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
