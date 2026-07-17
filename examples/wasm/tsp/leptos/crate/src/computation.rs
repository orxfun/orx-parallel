use crate::locations::Location;
use ordered_float::OrderedFloat;
use orx_parallel::*;
use rand::prelude::*;
use rand::rngs::SmallRng;

pub struct SearchRunOutput {
    pub best: Option<Solution>,
    pub iterations: usize,
}

pub struct Solution {
    pub tour: Vec<usize>,
    pub distance: f64,
}

pub fn run_search_sequential(
    iterations: usize,
    seed: u64,
    locations: &[Location],
) -> SearchRunOutput {
    let mut rng = SmallRng::seed_from_u64(seed);
    let best = (0..iterations)
        .map(|_| create_tour(&mut rng, locations))
        .min_by_key(|x| OrderedFloat::from(x.distance));

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
        .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))
        .map(|rng, _| create_tour(rng, locations))
        .min_by_key(|_, x| OrderedFloat::from(x.distance));

    SearchRunOutput { best, iterations }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_and_parallel_search_agree() {
        let locations = crate::locations::create_locations(123, 20);
        let iterations = 32;
        let seed = 12345;

        let sequential = run_search_sequential(iterations, seed, &locations);
        let parallel = run_search_parallel(iterations, seed, 2, 1, &locations);

        assert_eq!(sequential.iterations, iterations);
        assert_eq!(parallel.iterations, iterations);

        let sequential_best = sequential
            .best
            .expect("sequential search should find a tour");
        let parallel_best = parallel.best.expect("parallel search should find a tour");

        assert_eq!(sequential_best.tour, parallel_best.tour);
        assert_eq!(sequential_best.distance, parallel_best.distance);
    }
}
