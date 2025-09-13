mod std_default_pool;
mod std_scoped_threads;

// #[cfg(feature = "threadpool")]
// mod impl_threadpool;

// #[cfg(feature = "scoped_threadpool")]
// mod impl_scoped_threadpool;

// #[cfg(feature = "rayon")]
// mod impl_rayon_threadpool;

pub use std_default_pool::StdDefaultPool;
