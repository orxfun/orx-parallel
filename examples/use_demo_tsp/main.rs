mod locations;
mod par_with_use;
mod par_without_use;
mod rand_utils;

use crate::{
    locations::locations, par_with_use::run_search_parallel_use_mut,
    par_without_use::run_search_parallel_immutable,
};

fn main() {
    let iterations = 10000;
    let threads = 4;
    let num_cities = 50;
    let seed = 42;

    let locations: Vec<_> = locations(num_cities);
    let x = run_search_parallel_immutable(&locations, iterations, seed, threads);
    dbg!(x);

    let x = run_search_parallel_use_mut(&locations, iterations, seed, threads);
    dbg!(x);
}
