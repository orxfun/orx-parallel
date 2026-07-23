use crate::locations::Location;
use crate::rand_utils::{rng_from_seed, seed_for};
use core::cmp::Ordering::Equal;
use orx_parallel::*;
use rand::prelude::*;

struct ThreadData {
    min_cost: f64,
    temp_tour: Vec<usize>,
    best_tour: Vec<usize>,
}

impl ThreadData {
    fn new(num_cities: usize) -> Self {
        Self {
            min_cost: f64::MAX,
            temp_tour: (0..num_cities).collect(),
            best_tour: (0..num_cities).collect(),
        }
    }

    fn evaluate_temp_tour(&mut self, cost: f64) {
        if cost < self.min_cost {
            // temp tour becomes the best tour
            self.min_cost = cost;
            core::mem::swap(&mut self.temp_tour, &mut self.best_tour);
        }
    }
}

pub fn run_search_parallel_use_mut(
    locations: &[Location],
    iterations: usize,
    seed: u64,
    threads: usize,
) -> Option<(Vec<usize>, f64)> {
    let mut data = UseVec::new(|_| ThreadData::new(locations.len()));

    let par = (0..iterations).into_par().num_threads(threads);
    let par = par.use_vec(&mut data);
    par.for_each(|data, k| {
        let cost = search_candidate(locations, seed_for(seed, k), &mut data.temp_tour);
        data.evaluate_temp_tour(cost);
    });

    data.into_vec()
        .into_iter()
        .min_by(|x, y| x.min_cost.partial_cmp(&y.min_cost).unwrap_or(Equal))
        .map(|x| (x.best_tour, x.min_cost))
}

fn search_candidate(locations: &[Location], seed: u64, tour: &mut [usize]) -> f64 {
    random_tour(seed, tour);
    two_opt_improve(locations, tour);

    Location::tour_distance(locations, tour)
}

fn random_tour(seed: u64, tour: &mut [usize]) {
    for (i, x) in tour.iter_mut().enumerate() {
        *x = i
    }
    let mut rng = rng_from_seed(seed);
    tour.shuffle(&mut rng);
}

fn two_opt_improve(locations: &[Location], tour: &mut [usize]) {
    let edge_distance = |i: usize, j: usize| locations[i].distance_to(locations[j]);

    let n = tour.len();
    if n < 4 {
        return;
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
}
