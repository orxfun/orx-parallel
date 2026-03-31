#[cfg(feature = "std")]
mod std_default_pool;

#[cfg(feature = "std")]
pub use std_default_pool::StdDefaultPool;
