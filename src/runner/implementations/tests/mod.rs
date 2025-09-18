#[cfg(feature = "rayon")]
mod rayon;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;

#[cfg(feature = "scoped-pool")]
mod scoped_pool;

#[cfg(feature = "std")]
mod std;

mod sequential;

mod utils;

use utils::run_map;
