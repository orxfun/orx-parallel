#[cfg(feature = "long-tests")]
#[cfg(test)]
mod tests;

mod r#use;
mod use_fun;
mod use_pair;
mod use_slice;
mod use_vec;
mod use_vec_som;

pub use r#use::Use;
pub use use_fun::UseFun;
pub use use_slice::UseSlice;
pub use use_vec::UseVec;
