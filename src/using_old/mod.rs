mod collect_into;
/// Module containing variants of parallel iterators using a mutable variable.
pub mod computational_variants;
mod computations;
mod runner;
mod u_par_iter;
mod using_variants;

pub(crate) use collect_into::UParCollectIntoCoreOld;
pub use u_par_iter::ParIterUsingOld;
pub use using_variants::{Using, UsingClone, UsingFun};
