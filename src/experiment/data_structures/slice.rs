use crate::experiment::data_structures::{
    slice_iter_ptr::SliceIterPtr, slice_iter_ptr_dst::SliceIterPtrDst,
    slice_iter_ptr_src::SliceIterPtrSrc,
};
use alloc::vec::Vec;
use core::{marker::PhantomData, ptr::slice_from_raw_parts};

/// A raw slice of contiguous data.
pub struct Slice<'a, T: 'a> {
    raw: *const [T],
    phantom: PhantomData<&'a ()>,
}

impl<'a, T: 'a> From<&[T]> for Slice<'a, T> {
    fn from(value: &[T]) -> Self {
        Self::new(value.as_ptr(), value.len())
    }
}

impl<'a, T: 'a> Slice<'a, T> {
    /// Creates a new raw slice.
    #[inline(always)]
    pub fn new(data: *const T, len: usize) -> Self {
        let raw = slice_from_raw_parts(data, len);
        let phantom = PhantomData;
        Self { raw, phantom }
    }

    pub fn empty() -> Self {
        // SAFETY: satisfies (i)
        unsafe { Self::new(core::ptr::null(), 0) }
    }

    pub fn from_vec_capacity(vec: &Vec<T>) -> Self {
        // SAFETY: constructing from a valid vec allocation.
        unsafe { Self::new(vec.as_ptr(), vec.capacity()) }
    }

    pub(super) fn data(&self) -> *const T {
        self.raw as *const T
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// # SAFETY
    ///
    /// - (i) `self` and `src` must have the same lengths.
    /// - (ii) `self` and `src` must not be overlapping.
    pub unsafe fn copy_from_nonoverlapping(&self, src: &Self) {
        debug_assert_eq!(self.len(), src.len());

        // SAFETY: (i) within bounds and (ii) slices do not overlap
        let dst = self.raw as *mut T;
        unsafe { dst.copy_from_nonoverlapping(src.raw as *const T, self.len()) };
    }
}
