use crate::experiment::data_structures::slice::{Slice, SliceSafe};
use crate::experiment::data_structures::slice_iter_ptr_dst::SliceIterPtrDst;
use alloc::vec::Vec;

/// A raw slice of contiguous data with un-initialized values.
///
/// # SAFETY
///
/// While constructing this slice, we must guarantee that none of the elements of it
/// is initialized since they will be overwritten.
///
/// This is a write-only slice.
/// The caller must make sure that there is no other concurrent reads or writes to this slice.
pub struct SliceDst<'a, T>(Slice<'a, T>);

impl<'a, T> SliceDst<'a, T> {
    /// Destructs the slice wrapper and returns the underlying raw slice
    /// that it is created with.
    pub fn destruct(self) -> *const [T] {
        self.0.destruct()
    }

    /// Clones the destination slice.
    ///
    /// # SAFETY
    ///
    /// The purpose of destination slice is to mutate the underlying memory.
    /// Therefore, cloning is marked as unsafe.
    ///
    /// - (i) assuming the clone will be used to mutate the memory, caller must
    ///   ensure that `&self` will not be used.
    pub unsafe fn clone(&self) -> Self {
        Self(self.0.clone())
    }

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
    /// # Panics
    ///
    /// - (i) if `vec.len()` is not zero.
    ///
    /// # SAFETY
    ///
    /// This slice cannot outlive the `vec` it is created for due to the lifetime relation.
    pub fn from_vec(vec: &'a mut Vec<T>) -> Self {
        assert!(vec.is_empty());

        // SAFETY: constructing with contiguous un-initialized elements
        unsafe { Self::new(vec.as_ptr(), vec.capacity()) }
    }

    /// Length of the slice.
    #[inline(always)]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a safe wrapper over this slice.
    #[inline(always)]
    pub fn core(&self) -> SliceSafe<'_, 'a, T> {
        self.into()
    }

    /// Converts the destination slice into a destination iterator.
    pub fn into_iter(self) -> SliceIterPtrDst<'a, T> {
        SliceIterPtrDst::new(self)
    }

    /// Creates two slices from this slice:
    ///
    /// - first slice for positions [0..position)
    /// - second slice for positions [position..]
    ///
    /// # SAFETY
    ///
    /// - (i) `position` must be less than or equal to `self.len()`
    pub unsafe fn split_at_unchecked(self, position: usize) -> [Self; 2] {
        // SAFETY: req't (i) is satisfied by cond'n (i)
        unsafe { self.0.split_at_unchecked(position) }.map(Self)
    }
}

impl<'c, 'a, T: 'a> From<&'c SliceDst<'a, T>> for SliceSafe<'c, 'a, T> {
    fn from(value: &'c SliceDst<'a, T>) -> Self {
        SliceSafe::new(&value.0)
    }
}
