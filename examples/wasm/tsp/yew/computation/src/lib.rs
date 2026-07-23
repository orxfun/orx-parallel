mod locations;
mod solver;

pub use locations::{Location, create_locations};
pub use solver::{Solution, run_search_parallel, run_search_sequential};
