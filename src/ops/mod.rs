#[cfg(feature = "long-tests")]
#[cfg(test)]
mod tests;

mod extend;
mod sum;

pub use extend::ParExtend;
pub use sum::Sum;
