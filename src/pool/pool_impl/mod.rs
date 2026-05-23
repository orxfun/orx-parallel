#[cfg(feature = "std")]
mod std_default_pool;

#[cfg(feature = "std")]
pub use std_default_pool::StdDefaultPool;

#[cfg(not(feature = "std"))]
mod sequential;
#[cfg(not(feature = "std"))]
pub use sequential::SequentialPool;

#[cfg(feature = "rayon-core")]
mod rayon_core;
