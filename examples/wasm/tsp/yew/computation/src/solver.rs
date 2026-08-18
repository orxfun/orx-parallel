use crate::locations::Location;
use ordered_float::OrderedFloat;
use orx_parallel::*;
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct Solution {
    pub tour: Vec<usize>,
    pub distance: f64,
}

pub fn run_search(
    iterations: usize,
    seed: u64,
    threads: usize,
    chunk_size: usize,
    locations: &[Location],
) -> Option<Solution> {
    (0..iterations)
        .into_par()
        .chunk_size(chunk_size)
        .num_threads(threads)
        .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))
        .map(|rng, _| create_tour(rng, locations))
        .min_by_key(|_, x| OrderedFloat(x.distance))
}

fn create_tour(rng: &mut impl Rng, locations: &[Location]) -> Solution {
    let tour = random_tour(rng, locations.len());
    let tour = two_opt_improve(locations, tour);
    let distance = Location::tour_distance(locations, &tour);
    Solution { tour, distance }
}

fn random_tour(rng: &mut impl Rng, num_cities: usize) -> Vec<usize> {
    let mut tour: Vec<usize> = (0..num_cities).collect();
    tour.shuffle(rng);
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
