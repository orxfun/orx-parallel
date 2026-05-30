#[cfg(test)]
mod tests;

mod new_use;
mod use_clone;
mod use_fun;
mod use_slice;
mod use_vec;
mod using;

pub use new_use::Use;
pub use use_clone::UseClone;
pub use use_fun::UseFun;
pub use use_slice::UseSlice;
pub use use_vec::UseVec;
pub use using::Using;
