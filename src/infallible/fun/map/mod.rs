mod r#enum;
mod fn_impl;
mod fn_trait;

pub use r#enum::MapEnum;
pub use fn_impl::{FnCloned, FnCopied, FnIns, FnMap};
pub use fn_trait::Map;
