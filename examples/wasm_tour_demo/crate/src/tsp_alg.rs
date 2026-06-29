use crate::locations::{Location, location_for};
use core::cmp::Ordering::Equal;
use orx_parallel::{IntoParIter, Par};
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct SearchRunOutput {
    pub best: Option<(Vec<usize>, f64)>,
    pub iterations: usize,
    pub elapsed_ms: f64,
}

pub fn run_search_parallel(
    iterations: usize,
    seed: u64,
    threads: usize,
    num_cities: usize,
    start_index: u64,
) -> SearchRunOutput {
    let start = js_sys::Date::now();
    let pool = orx_parallel::Pool::wasm_web(threads);

    let best = (0..iterations)
        .into_par()
        .pool(pool)
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), num_cities))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

    let elapsed_ms = js_sys::Date::now() - start;
    SearchRunOutput {
        best,
        iterations,
        elapsed_ms,
    }
}

pub fn run_search_sequential(
    iterations: usize,
    seed: u64,
    num_cities: usize,
    start_index: u64,
) -> SearchRunOutput {
    let start = js_sys::Date::now();

    let best = (0..iterations)
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), num_cities))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

    let elapsed_ms = js_sys::Date::now() - start;
    SearchRunOutput {
        best,
        iterations,
        elapsed_ms,
    }
}

fn search_candidate(seed: u64, k: u64, num_cities: usize) -> (Vec<usize>, f64) {
    let tour = random_tour(seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15), num_cities);
    let tour = two_opt_improve(tour);
    let distance = tour_distance(&tour);
    (tour, distance)
}

fn random_tour(seed: u64, num_cities: usize) -> Vec<usize> {
    let mut tour: Vec<usize> = (0..num_cities).collect();
    let mut rng = SmallRng::seed_from_u64(seed);
    tour.shuffle(&mut rng);
    tour
}

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

fn euclidean(a: Location, b: Location) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn edge_distance(i: usize, j: usize) -> f64 {
    euclidean(location_for(i), location_for(j))
}

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
