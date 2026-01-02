#[cfg(test)]
mod tests;

mod map_result;
mod par_result;
mod xap_result;

pub use map_result::ParMapResult;
pub use par_result::ParResult;
pub use xap_result::ParXapResult;
