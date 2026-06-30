mod locations;
mod par_with_use;
mod par_without_use;
mod rand_utils;

use crate::{locations::location_for, par_without_use::run_search_parallel};

fn main() {
    let iterations = 10000;
    let threads = 4;
    let num_cities = 5;
    let seed = 42;

    let locations: Vec<_> = (0..num_cities).map(location_for).collect();

    let x = run_search_parallel(&locations, iterations, seed, threads);
    dbg!(x);
}
