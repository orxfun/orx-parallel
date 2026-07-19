#![cfg(target_arch = "wasm32")]

use computation::Location;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen_test::*;
use wasm_bindings::{RunResult, locations, run_search};

#[cfg(target_feature = "atomics")]
use wasm_bindings::init_parallel_runtime;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn locations_returns_the_requested_number_of_points() {
    let value = locations(7, 4).expect("locations should serialize");
    let locations: Vec<Location> = from_value(value).expect("locations should deserialize");

    assert_eq!(locations.len(), 4);
    assert!(
        locations
            .iter()
            .all(|location| (-50.0..50.0).contains(&location.x))
    );
    assert!(
        locations
            .iter()
            .all(|location| (-50.0..50.0).contains(&location.y))
    );
}

#[wasm_bindgen_test]
fn run_search_returns_a_tour_summary() {
    let locations = vec![
        Location { x: 0.0, y: 0.0 },
        Location { x: 1.0, y: 0.0 },
        Location { x: 1.0, y: 1.0 },
        Location { x: 0.0, y: 1.0 },
    ];
    let locations_val = to_value(&locations).expect("locations should serialize");

    let result = run_search(false, 1, 7, 1, 1, locations_val).expect("run_search should succeed");
    let result: RunResult = from_value(result).expect("result should deserialize");

    assert_eq!(result.iterations, 1);
    assert_eq!(result.best_tour.len(), locations.len());
    assert!(result.best_distance > 0.0);
    assert!(result.elapsed_ms >= 0.0);
    assert_is_tour(&result.best_tour, locations.len());
}

#[wasm_bindgen_test]
#[cfg(target_feature = "atomics")]
fn init_parallel_runtime_returns_a_promise() {
    let _promise = init_parallel_runtime(1);
}

fn assert_is_tour(tour: &[usize], expected_len: usize) {
    assert_eq!(tour.len(), expected_len, "incorrect tour length");
    let mut sorted = tour.to_vec();
    sorted.sort();
    for (i, x) in sorted.iter().copied().enumerate() {
        assert_eq!(i, x, "invalid tour");
    }
}
