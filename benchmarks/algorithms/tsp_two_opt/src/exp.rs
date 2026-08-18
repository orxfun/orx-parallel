use crate::{alg::Method, input::InputVariant};
use ordered_float::OrderedFloat;
use orx_criterion::Experiment;
use rand::prelude::*;
use rand::rngs::SmallRng;

const X_RANGE: [f64; 2] = [-50.0, 50.0];
const Y_RANGE: [f64; 2] = [-50.0, 50.0];

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
pub struct Solution {
    pub tour: Vec<usize>,
    pub distance: f64,
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        (self.distance - other.distance).abs() < 1e-9 && self.tour == other.tour
    }
}

fn create_locations(seed: u64, num_cities: usize) -> Vec<Location> {
    let mut rng = SmallRng::seed_from_u64(seed);
    (0..num_cities)
        .map(|_| Location {
            x: rng.random_range(X_RANGE[0]..X_RANGE[1]),
            y: rng.random_range(Y_RANGE[0]..Y_RANGE[1]),
        })
        .collect()
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

fn create_tour(rng: &mut impl Rng, locations: &[Location]) -> Solution {
    let tour = random_tour(rng, locations.len());
    let tour = two_opt_improve(locations, tour);
    let distance = Location::tour_distance(locations, &tour);
    Solution { tour, distance }
}

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = (Vec<Location>, usize);
    type Output = Solution;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        const SEED: u64 = 42;
        let locations = create_locations(SEED, input_variant.num_cities);
        (locations, input_variant.iterations)
    }

    fn execute(
        &mut self,
        _input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let (locations, iterations) = input;
        match alg_variant {
            Method::Seq => run_seq(locations, *iterations),
            Method::Rayon => run_rayon(locations, *iterations),
            Method::OrxOnce => run_orx(locations, *iterations),
            Method::OrxBasic => run_orx(locations, *iterations),
            Method::OrxRayon => run_orx(locations, *iterations),
        }
    }

    fn validate_output(
        &self,
        _input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let (locations, _) = input;
        let num_cities = locations.len();

        assert_eq!(output.tour.len(), num_cities,);

        let mut visited = vec![false; num_cities];
        for &city in &output.tour {
            assert!(!visited[city],);
            visited[city] = true;
        }

        // Verify distance is calculated correctly
        let calculated_distance = Location::tour_distance(locations, &output.tour);
        assert!(
            (calculated_distance - output.distance).abs() < 1e-9,
            "Tour distance mismatch: calculated {}, but Solution has {}",
            calculated_distance,
            output.distance
        );
    }
}

fn run_seq(locations: &[Location], iterations: usize) -> Solution {
    let mut rng = SmallRng::seed_from_u64(0);
    (0..iterations)
        .map(|_| create_tour(&mut rng, locations))
        .min_by_key(|x| OrderedFloat(x.distance))
        .expect("at least one iteration")
}

fn run_rayon(locations: &[Location], iterations: usize) -> Solution {
    use rayon::prelude::*;
    (0..iterations)
        .into_par_iter()
        .map(|i| {
            let mut rng = SmallRng::seed_from_u64(i as u64);
            create_tour(&mut rng, locations)
        })
        .min_by_key(|x| OrderedFloat(x.distance))
        .expect("at least one iteration")
}

fn run_orx(locations: &[Location], iterations: usize) -> Solution {
    use orx_parallel::*;
    (0..iterations)
        .into_par()
        .use_new(|i| SmallRng::seed_from_u64(i as u64))
        .map(|rng, _| create_tour(rng, locations))
        .min_by_key(|_, x| OrderedFloat(x.distance))
        .expect("at least one iteration")
}
