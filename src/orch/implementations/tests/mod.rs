#[cfg(feature = "rayon")]
mod rayon;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;

mod utils;
use utils::run_map;
