#[cfg(feature = "long-tests")]
#[cfg(test)]
mod tests;

mod extend;
mod sum;

pub use extend::ParExtendOld;
pub use sum::Sum;
