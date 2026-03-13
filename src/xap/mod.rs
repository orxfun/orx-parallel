pub mod count;
// mod faker;
pub mod fun;
mod xap_implementors;
pub mod xap_iter;
mod xap_trait;

pub use xap_implementors::{F, FilMap, FlaMap, Id, M};
pub use xap_trait::{Xap, XapCloned, XapCopied};
