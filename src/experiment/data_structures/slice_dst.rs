use crate::experiment::data_structures::slice::Slice;
use alloc::vec::Vec;

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

    /// Creates a new slice for the entire capacity of the vector.
    ///
    /// # PANICS
    ///
    /// - (i) if `vec.len()` is not zero.
    ///
    /// # SAFETY
    ///
    /// This slice cannot outlive the `vec` it is created for due to the lifetime relation.
    pub fn from_vec(vec: &'a Vec<T>) -> Self {
        assert_eq!(vec.len(), 0);

        // SAFETY: constructing with contiguous un-initialized elements
        unsafe { Self::new(vec.as_ptr(), vec.capacity()) }
    }
}
