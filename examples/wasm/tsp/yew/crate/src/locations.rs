use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::Serialize;

const MIN_CITIES: usize = 5;
const MAX_CITIES: usize = 200;

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
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

pub fn locations(seed: u64, num_cities: u32) -> Vec<Location> {
    let num_cities = clamp_num_cities(num_cities);
    let mut rng = SmallRng::seed_from_u64(seed);

    (0..num_cities)
        .map(|_| Location {
            x: rng.random_range(-50.0..50.0),
            y: rng.random_range(-50.0..50.0),
        })
        .collect()
}

pub(crate) fn clamp_num_cities(num_cities: u32) -> usize {
    (num_cities as usize).clamp(MIN_CITIES, MAX_CITIES)
}
