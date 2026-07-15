use core::cmp::Ordering::Equal;
use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::Serialize;

pub const MIN_CITIES: usize = 5;
pub const MAX_CITIES: usize = 200;

pub const RUN_FIB: bool = false;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Location {
    pub x: f64,
    pub y: f64,
}

impl Location {
    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn tour_distance(locations: &[Location], tour: &[usize]) -> f64 {
        match (tour.first(), tour.last()) {
            (Some(&first), Some(&last)) => {
                let middle_distance: f64 = tour
                    .windows(2)
                    .map(|w| locations[w[0]].distance_to(locations[w[1]]))
                    .sum();
                let closing_distance = locations[last].distance_to(locations[first]);
                middle_distance + closing_distance
            }
            _ => 0.0,
        }
    }
}

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

pub fn search_candidate(seed: u64, k: u64, locations: &[Location]) -> (Vec<usize>, f64) {
    let tour = random_tour(
        seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        locations.len(),
    );
    let tour = two_opt_improve(locations, tour);
    let distance = Location::tour_distance(locations, &tour);
    (tour, distance)
}

pub fn locations(num_cities: u32) -> Vec<Location> {
    let num_cities = clamp_num_cities(num_cities);
    (0..num_cities).map(location_for).collect()
}

pub fn clamp_num_cities(num_cities: u32) -> usize {
    (num_cities as usize).clamp(MIN_CITIES, MAX_CITIES)
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

fn location_for(idx: usize) -> Location {
    let sx = split_mix_64((idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let sy = split_mix_64((idx as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));

    let x = 100.0 * to_unit_f64(sx) - 50.0;
    let y = 100.0 * to_unit_f64(sy) - 50.0;
    Location { x, y }
}

fn split_mix_64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn to_unit_f64(x: u64) -> f64 {
    let v = x >> 11;
    (v as f64) * (1.0 / ((1u64 << 53) as f64))
}
