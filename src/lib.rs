mod core;
mod into_par;
mod par;

pub use core::params::Params;
pub use into_par::IntoPar;
pub use par::collect_into::par_map_collect_into::ParMapCollectInto;
pub use par::{
    par_empty::Par, par_fil::ParFilter, par_map::ParMap, par_map_fil::ParMapFilter, reduce::Reduce,
};
