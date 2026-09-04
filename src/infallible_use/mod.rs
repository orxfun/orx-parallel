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
mod xap_iter;
pub mod xap_variants;

pub use par::ParUse;
pub use par_enum::EnumerateParUse;
pub use par_iter::ParUseIter;
pub use xap::{
    FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, XapUse, XapUseBin, XapUseOne,
};
pub use xap_enum::XapUseEnumByInput;
pub use xap_iter::XapUseIter;
