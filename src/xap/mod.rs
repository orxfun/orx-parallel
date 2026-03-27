mod count;
pub mod fallible;
mod fun;
mod xap_implementors;
mod xap_trait;

pub use xap_implementors::{FilMap, FlaMap, Id, M};
pub use xap_trait::{Xap, XapCloned, XapCopied};
