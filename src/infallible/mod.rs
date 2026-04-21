#[cfg(test)]
mod tests;

pub mod fun;
mod par;
mod par_iter;
mod par_iter_core;
mod par_iter_enum;
mod par_runner;
mod thread_execution;
mod xap;
mod xap_enum;
pub mod xap_variants;

pub use par::Par;
pub use par_iter::ParIter;
pub use par_iter_core::ParIterCore;
pub use par_iter_enum::ParIterEnumarable;
pub use par_runner::ParRunnerInfallible;
pub use xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Xap, XapBin, XapOne};
pub use xap_enum::XapEnumByInput;
