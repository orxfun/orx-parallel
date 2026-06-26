#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
use orx_parallel::{IntoParIter, Par};
#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
use rand::prelude::*;
#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
use rand::rngs::SmallRng;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, Serialize)]
struct Location {
    x: f64,
    y: f64,
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[derive(Debug, Serialize)]
struct RunResult {
    best_tour: Vec<usize>,
    best_distance: f64,
    iterations: usize,
    elapsed_ms: f64,
}

const LOCATIONS: [Location; 12] = [
    Location { x: 3.0, y: 7.0 },
    Location { x: 9.0, y: 5.0 },
    Location { x: 3.0, y: 0.0 },
    Location { x: 5.0, y: -3.0 },
    Location { x: 12.0, y: 4.0 },
    Location { x: 8.0, y: 9.0 },
    Location { x: 6.0, y: 17.0 },
    Location { x: 0.0, y: 11.0 },
    Location { x: -3.0, y: 2.0 },
    Location { x: 14.0, y: -2.0 },
    Location { x: 2.0, y: 14.0 },
    Location { x: -4.0, y: 9.0 },
];

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
pub fn locations() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&LOCATIONS)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

#[wasm_bindgen]
pub fn run_best_tour(iterations: u32, seed: u64, threads: u32) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, threads);
        return Err(JsValue::from_str(
            "run_best_tour requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let threads = threads.max(1) as usize;

        let start = js_sys::Date::now();

        let pool = orx_parallel::Pool::wasm_web(threads);

        let best = (0..iterations)
            .into_par()
            .pool(pool)
            .map(|k| random_tour(seed ^ (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)))
            .map(|tour| {
                let distance = tour_distance(&tour);
                (tour, distance)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));

        let elapsed_ms = js_sys::Date::now() - start;

        let (best_tour, best_distance) = match best {
            Some(v) => v,
            None => {
                return Err(JsValue::from_str(
                    "no tour could be generated (unexpected empty search)",
                ));
            }
        };

        let result = RunResult {
            best_tour,
            best_distance,
            iterations,
            elapsed_ms,
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("failed to serialize result: {e}")))
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn random_tour(seed: u64) -> Vec<usize> {
    let mut tour: Vec<usize> = (0..LOCATIONS.len()).collect();
    let mut rng = SmallRng::seed_from_u64(seed);
    tour.shuffle(&mut rng);
    tour
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn tour_distance(tour: &[usize]) -> f64 {
    if tour.len() <= 1 {
        return 0.0;
    }

    let mut sum = 0.0;
    for w in tour.windows(2) {
        let a = LOCATIONS[w[0]];
        let b = LOCATIONS[w[1]];
        sum += euclidean(a, b);
    }

    let first = LOCATIONS[tour[0]];
    let last = LOCATIONS[*tour.last().expect("tour has at least one location")];
    sum + euclidean(last, first)
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn euclidean(a: Location, b: Location) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
