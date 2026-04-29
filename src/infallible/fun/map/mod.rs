mod basic;
mod cloned_copied;
mod r#enum;
mod flatten;
mod fn_trait;
mod inspect;

pub use basic::FnMap;
pub use cloned_copied::{FnCloned, FnCopied};
pub use r#enum::MapEnum;
pub use flatten::FnFlatten;
pub use fn_trait::Map;
pub use inspect::FnIns;
