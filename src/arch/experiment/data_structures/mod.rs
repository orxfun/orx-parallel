#[cfg(test)]
mod tests;

/// A raw slice of contiguous data.
pub mod slice;
/// A raw slice of contiguous data with un-initialized values.
pub mod slice_dst;
/// Core structure for iterators over contiguous slices of data.
pub mod slice_iter_ptr;
/// Iterator over a slice of data that will be completely filled with values before the iterator is consumed.
pub mod slice_iter_ptr_dst;
/// Iterator over a slice of data that will be completely copied to another slice before the iterator is consumed.
pub mod slice_iter_ptr_src;
/// A raw slice of contiguous data with initialized values.
pub mod slice_src;
