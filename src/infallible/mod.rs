#[cfg(feature = "long-tests")]
#[cfg(test)]
mod tests;

pub mod fun;
mod par;
mod par_core;
mod par_enum;
mod par_iter;
mod par_runner;
mod thread_execution;
mod xap;
mod xap_enum;
pub mod xap_variants;

pub use par::Par;
pub use par_core::ParCore;
pub use par_enum::EnumeratePar;
pub use par_iter::ParIter;
pub use par_runner::ParRunnerInfallible;
pub use xap::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Xap, XapBin, XapOne};
pub use xap_enum::XapEnumByInput;
