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

const MIN_CITIES: usize = 5;
const MAX_CITIES: usize = 200;

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
    let num_cities = clamp_num_cities(num_cities);
    let locations: Vec<Location> = (0..num_cities).map(location_for).collect();
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

#[wasm_bindgen]
pub fn run_best_tour(
    iterations: u32,
    seed: u64,
    threads: u32,
    num_cities: u32,
) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, threads, num_cities);
        return Err(JsValue::from_str(
            "run_best_tour requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let threads = threads.max(1) as usize;
        let num_cities = clamp_num_cities(num_cities);
        run_search_parallel(iterations, seed, threads, num_cities)
    }
}

#[wasm_bindgen]
pub fn run_best_tour_seq(iterations: u32, seed: u64, num_cities: u32) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, num_cities);
        return Err(JsValue::from_str(
            "run_best_tour_seq requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let num_cities = clamp_num_cities(num_cities);
        run_search_sequential(iterations, seed, num_cities)
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn run_search_parallel(
    iterations: usize,
    seed: u64,
    threads: usize,
    num_cities: usize,
) -> Result<JsValue, JsValue> {
    let start = js_sys::Date::now();
    let pool = orx_parallel::Pool::wasm_web(threads);

    let best = (0..iterations)
        .into_par()
        .pool(pool)
        .map(|k| search_candidate(seed, k as u64, num_cities))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));

    let elapsed_ms = js_sys::Date::now() - start;
    run_result_to_js(best, iterations, elapsed_ms)
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn run_search_sequential(
    iterations: usize,
    seed: u64,
    num_cities: usize,
) -> Result<JsValue, JsValue> {
    let start = js_sys::Date::now();

    let best = (0..iterations)
        .map(|k| search_candidate(seed, k as u64, num_cities))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));

    let elapsed_ms = js_sys::Date::now() - start;
    run_result_to_js(best, iterations, elapsed_ms)
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn search_candidate(seed: u64, k: u64, num_cities: usize) -> (Vec<usize>, f64) {
    let tour = random_tour(seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15), num_cities);
    let tour = two_opt_improve(tour);
    let distance = tour_distance(&tour);
    (tour, distance)
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn run_result_to_js(
    best: Option<(Vec<usize>, f64)>,
    iterations: usize,
    elapsed_ms: f64,
) -> Result<JsValue, JsValue> {
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

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn random_tour(seed: u64, num_cities: usize) -> Vec<usize> {
    let mut tour: Vec<usize> = (0..num_cities).collect();
    let mut rng = SmallRng::seed_from_u64(seed);
    tour.shuffle(&mut rng);
    tour
}

fn clamp_num_cities(num_cities: u32) -> usize {
    (num_cities as usize).clamp(MIN_CITIES, MAX_CITIES)
}

fn location_for(idx: usize) -> Location {
    // Use a deterministic spiral-like layout for any requested city count.
    let t = idx as f64;
    let theta = t * 0.618_033_988_749_894_9 * core::f64::consts::TAU;
    let radius = 8.0 + 2.4 * t.sqrt();
    Location {
        x: radius * theta.cos(),
        y: radius * theta.sin(),
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn two_opt_improve(mut tour: Vec<usize>) -> Vec<usize> {
    let n = tour.len();
    if n < 4 {
        return tour;
    }

    let mut improved = true;
    while improved {
        improved = false;

        for i in 0..(n - 1) {
            let a = tour[i];
            let b = tour[(i + 1) % n];

            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }

                let c = tour[j];
                let d = tour[(j + 1) % n];

                let current = edge_distance(a, b) + edge_distance(c, d);
                let swapped = edge_distance(a, c) + edge_distance(b, d);

                if swapped + 1e-12 < current {
                    tour[(i + 1)..=j].reverse();
                    improved = true;
                    break;
                }
            }

            if improved {
                break;
            }
        }
    }

    tour
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn edge_distance(i: usize, j: usize) -> f64 {
    euclidean(location_for(i), location_for(j))
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn tour_distance(tour: &[usize]) -> f64 {
    if tour.len() <= 1 {
        return 0.0;
    }

    let mut sum = 0.0;
    for w in tour.windows(2) {
        let a = location_for(w[0]);
        let b = location_for(w[1]);
        sum += euclidean(a, b);
    }

    let first = location_for(tour[0]);
    let last = location_for(*tour.last().expect("tour has at least one location"));
    sum + euclidean(last, first)
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn euclidean(a: Location, b: Location) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
