mod collect_into;
mod computational_variants;
mod executor;
mod u_par_iter;
mod using_variants;

pub(crate) use collect_into::UParCollectIntoCore;
pub(crate) use computational_variants::{UPar, UParMap, UParXap};
pub use u_par_iter::ParIterUsing;
pub use using_variants::{Using, UsingClone, UsingFun};
