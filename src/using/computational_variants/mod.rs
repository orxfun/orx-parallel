#[cfg(test)]
mod tests;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with None.
pub mod u_fallible_option;
/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub mod u_fallible_result;

mod u_map;
mod u_par;
mod u_xap;

pub use u_map::UParMap;
pub use u_par::UPar;
pub use u_xap::UParXap;
