#[cfg(test)]
mod tests;

pub mod fun;
mod par_iter;
mod par_runner;
pub mod sizes;
mod thread_execution;
mod xap;
mod xap_enum;
pub mod xap_variants;

pub use par_iter::Par;
pub use xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Xap, XapBin, XapOne};
pub use xap_enum::XapEnumByInput;
