use wasm_bindgen::prelude::*;

mod locations;
#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
mod tsp_alg;

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen(js_name = init_thread_pool)]
pub fn init_thread_pool_export(num_threads: u32) -> js_sys::Promise {
    orx_parallel::init_thread_pool(num_threads as usize)
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen(js_name = init_thread_pool)]
pub fn init_thread_pool_export(_num_threads: u32) -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "init_thread_pool is only available for wasm32 + atomics builds",
    ))
}

#[wasm_bindgen]
pub fn locations(num_cities: u32) -> Result<JsValue, JsValue> {
    locations::locations(num_cities)
}

#[wasm_bindgen]
pub fn run_best_tour(
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
            "run_best_tour requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let threads = threads.max(1) as usize;
        let num_cities = locations::clamp_num_cities(num_cities);
        tsp_alg::run_search_parallel(iterations, seed, threads, num_cities, start_index)
    }
}

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
        tsp_alg::run_search_sequential(iterations, seed, num_cities, start_index)
    }
}
