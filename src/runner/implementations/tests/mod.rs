#[cfg(feature = "pond")]
mod pond;

#[cfg(feature = "poolite")]
mod poolite;
#[cfg(feature = "yastl")]
mod yastl;

#[cfg(feature = "rayon")]
mod rayon;

#[cfg(feature = "scoped-pool")]
mod scoped_pool;

#[cfg(feature = "scoped_threadpool")]
mod scoped_threadpool;

#[cfg(feature = "std")]
mod std;

#[cfg(feature = "yastl")]
mod yastl;

mod sequential;

mod utils;

use utils::run_map;
