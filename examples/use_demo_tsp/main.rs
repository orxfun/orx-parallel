mod locations;
mod par_with_use;
mod par_without_use;
mod rand_utils;

use crate::{locations::locations, par_without_use::run_search_parallel};

fn main() {
    let iterations = 10000;
    let threads = 4;
    let num_cities = 5;
    let seed = 42;

    let locations: Vec<_> = locations(num_cities);
    let x = run_search_parallel(&locations, iterations, seed, threads);
    dbg!(x);
}
