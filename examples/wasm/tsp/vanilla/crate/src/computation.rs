use crate::locations::Location;
use core::cmp::Ordering::Equal;
use orx_parallel::*;
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct SearchRunOutput {
    pub best: Option<(Vec<usize>, f64)>,
    pub iterations: usize,
}

pub fn run_search_sequential(
    iterations: usize,
    seed: u64,
    locations: &[Location],
    start_index: u64,
) -> SearchRunOutput {
    let best = (0..iterations)
        .map(|k| search_candidate(seed, start_index.wrapping_add(k as u64), locations))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

    SearchRunOutput { best, iterations }
}

pub fn run_search_parallel(
    iterations: usize,
    seed: u64,
    threads: usize,
    chunk_size: usize,
    locations: &[Location],
) -> SearchRunOutput {
    let best = (0..iterations)
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(threads)
        .map(|k| search_candidate(seed, k as u64, locations))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

    SearchRunOutput { best, iterations }
}

fn search_candidate(seed: u64, k: u64, locations: &[Location]) -> (Vec<usize>, f64) {
    let tour = random_tour(
        seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        locations.len(),
    );
    let tour = two_opt_improve(locations, tour);
    let distance = Location::tour_distance(locations, &tour);
    (tour, distance)
}

fn random_tour(seed: u64, num_cities: usize) -> Vec<usize> {
    let mut tour: Vec<usize> = (0..num_cities).collect();
    let mut rng = SmallRng::seed_from_u64(seed);
    tour.shuffle(&mut rng);
    tour
}

fn two_opt_improve(locations: &[Location], mut tour: Vec<usize>) -> Vec<usize> {
    let edge_distance = |i: usize, j: usize| locations[i].distance_to(locations[j]);

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
