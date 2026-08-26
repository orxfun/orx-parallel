#[cfg(feature = "long-tests")]
#[cfg(test)]
mod tests;

/// Function transformations used in [Xap] computations.
///
/// [`Xap`]: crate::infallible::xap::Xap
pub mod fun;
mod par;
mod par_core;
mod par_enum;
mod par_iter;
mod par_runner;
mod recursive;
mod thread_execution;
mod xap;
mod xap_enum;
/// Variants of [Xap] computations.
///
/// [`Xap`]: crate::infallible::xap::Xap
pub mod xap_variants;

pub use par::Par;
pub(crate) use par_core::ParCore;
pub use par_enum::EnumeratePar;
pub use par_iter::ParIter;
pub(crate) use par_runner::ParRunnerInfallible;
pub use xap::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Xap, XapBin, XapOne};
pub use xap_enum::XapEnumByInput;
