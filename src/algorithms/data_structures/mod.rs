#[cfg(test)]
mod tests;

mod slice;
pub mod slice_iter;
mod slice_mut;

pub use slice::Slice;
pub use slice_mut::SliceMut;
