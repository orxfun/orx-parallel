use crate::experiment::data_structures::slice::Slice;

/// A raw slice of contiguous data with un-initialized values.
///
/// # SAFETY
///
/// While constructing this slice, we must guarantee that none of the elements of it
/// is initialized since they will be overwritten.
pub struct SliceDst<'a, T>(Slice<'a, T>);

impl<'a, T> SliceDst<'a, T> {
    /// Creates a new slice of un-initialized values.
    ///
    /// # SAFETY
    ///
    /// - (i) `data` to `data+len` must be contiguous memory of un-initialized elements
    pub unsafe fn new(data: *const T, len: usize) -> Self {
        Self(Slice::new(data, len))
    }
}
