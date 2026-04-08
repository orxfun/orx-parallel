pub mod fun;
mod par_runner;
mod sizes;
mod thread_execution;
mod using_var;
mod xap_use;
mod xap_use_enum;
pub mod xap_variants;

pub use xap_use::{Xap, XapBin, XapOne};
pub use xap_use_enum::XapEnumByInput;
