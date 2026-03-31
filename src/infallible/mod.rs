pub mod fun;
mod par_iter;
mod par_runner;
pub mod size;
mod thread_execution;
mod xap;
mod xap_enum;
pub mod xap_variants;

pub use par_iter::{Par, par};
pub use xap::{Xap, XapBin, XapOne};
pub use xap_enum::XapEnumByInput;
