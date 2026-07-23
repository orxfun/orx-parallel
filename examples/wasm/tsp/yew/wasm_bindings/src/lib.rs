use computation::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Result returned to the frontend after a search completes.
pub struct RunResult {
    pub best_tour: Vec<usize>,
    pub best_distance: f64,
    pub iterations: usize,
    pub elapsed_ms: f64,
}

#[wasm_bindgen]
/// Initializes the shared thread pool used by the parallel search path.
///
/// This function must be called once before invoking `run_search` when using the parallel mode.
#[allow(unused_variables)]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
/// Generates a random set of locations for a TSP instance.
pub fn locations(seed: u64, num_cities: u32) -> Result<JsValue, JsValue> {
    let locations = create_locations(seed, num_cities);
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

#[wasm_bindgen]
/// Runs the TSP search over the provided locations and returns the best tour.
///
/// Call `init_parallel_runtime` first if you plan to use the parallel search mode.
///
/// `locations` should be a JS array of objects shaped like `{ x: number, y: number }`.
///
/// Returns a JS object with `best_tour`, `best_distance`, `iterations`, and `elapsed_ms`;
/// where `best_tour` is an array of indices of the locations.
pub fn run_search(
    parallelize: bool,
    iterations: u32,
    seed: u64,
    threads: u32,
    chunk_size: u32,
    locations: JsValue,
) -> Result<JsValue, JsValue> {
    let locations: Vec<Location> = serde_wasm_bindgen::from_value(locations)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize locations: {e}")))?;
    let iterations = iterations.max(1) as usize;
    let threads = threads as usize;
    let chunk_size = chunk_size as usize;
    let started_at = js_sys::Date::now();
    let output = match parallelize {
        true => run_search_parallel(iterations, seed, threads, chunk_size, &locations),
        false => run_search_sequential(iterations, seed, &locations),
    };
    let elapsed_ms = js_sys::Date::now() - started_at;

    match output {
        Some(solution) => {
            let result = RunResult {
                best_tour: solution.tour,
                best_distance: solution.distance,
                iterations,
                elapsed_ms,
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {e}")))
        }
        None => Err(JsValue::from_str("Failed to create a tour")),
    }
}
