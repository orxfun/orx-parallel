mod collect_into;
mod computational_variants;
mod runner;
mod u_par_iter;
mod using_variants;

pub use collect_into::UParCollectIntoCore;
pub use computational_variants::{UPar, UParMap, UParXap};
pub use u_par_iter::ParIterUsing;
pub use using_variants::{Using, UsingClone, UsingFun};
