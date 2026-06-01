#[cfg(feature = "long-tests")]
#[cfg(test)]
pub mod par_col_into_test;

pub mod alg;
mod inf;
mod inf_use;
mod opt;
mod opt_use;
mod par_col_into;
mod res;
mod res_use;
mod vec2;

pub use par_col_into::ParCollectInto;
pub use vec2::Vec2;
