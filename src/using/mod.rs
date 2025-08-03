mod collect_into;
pub mod computational_variants;
mod computations;
mod runner;
mod u_par_iter;
mod using_variants;

pub(crate) use collect_into::UParCollectIntoCore;
pub use u_par_iter::ParIterUsing;
pub use using_variants::{Using, UsingClone, UsingFun};
