// #[cfg(test)]
// mod tests;

pub mod fun;
mod par_iter;
mod par_runner;
mod thread_execution;
pub mod using_var;
mod xap;
mod xap_enum;
pub mod xap_variants;

pub use par_iter::ParUsing;
pub use xap::Xap;
pub use xap_enum::XapEnumByInput;
