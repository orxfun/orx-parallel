pub mod fun;
mod par_iter;
mod par_iter_transform;
mod par_runner;
pub mod size;
mod thread_execution;
mod xap;
mod xap_cloned;
mod xap_copied;
pub mod xap_variants;

pub use xap::{Xap, XapBin, XapOne};
pub use xap_cloned::XapCloned;
pub use xap_copied::XapCopied;
