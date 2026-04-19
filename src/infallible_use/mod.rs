#[cfg(test)]
mod tests;

pub mod fun;
mod par;
mod par_iter;
mod par_runner;
mod sizes;
mod thread_execution;
mod use_var;
mod xap;
mod xap_enum;
pub mod xap_variants;

pub use par::ParUse;
pub use par_iter::ParUseIter;
pub use par_runner::ParRunnerInfallibleUse;
pub use sizes::SizeInfUse;
pub use use_var::{Use, UseClone, UseFun};
pub use xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, XapUse, XapUseBin, XapUseOne};
pub use xap_enum::XapUseEnumByInput;
