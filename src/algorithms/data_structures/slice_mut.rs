use crate::algorithms::data_structures::slice_iter::SliceIterDst;
use alloc::vec::Vec;
use core::{marker::PhantomData, ops::Range};

/// A slice of contiguous data.
///
/// Its lifetime is bound to the owner of the data.
pub struct SliceMut<'a, T> {
    data: *mut T,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

/// SAFETY: [`Slice`] containing a raw pointer is allowed to send across threads.
/// Since the lifetime of the slice is bound to the owner of the data, this pointer
/// will be valid.
unsafe impl<T: Send> Send for SliceMut<'_, T> {}

/// SAFETY: [`Slice`] containing a raw pointer is allowed to shared across threads.
/// Since the lifetime of the slice is bound to the owner of the data, this pointer
/// will be valid.
unsafe impl<T: Sync> Sync for SliceMut<'_, T> {}

impl<'a, T> From<&'a mut [T]> for SliceMut<'a, T> {
    #[inline(always)]
    fn from(value: &'a mut [T]) -> Self {
        Self::new(value.as_mut_ptr(), value.len())
    }
}

impl<'a, T> From<&'a mut Vec<T>> for SliceMut<'a, T> {
    fn from(value: &'a mut Vec<T>) -> Self {
        Self::new(value.as_mut_ptr(), value.capacity())
    }
}

impl<'a, T> SliceMut<'a, T> {
    #[inline(always)]
    pub(super) fn new(data: *mut T, len: usize) -> Self {
        Self {
            data,
            len,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub(super) fn data(&self) -> *const T {
        self.data
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn slice(&mut self, range: Range<usize>) -> Self {
        let data = unsafe { self.data.add(range.start) };
        let len = range.len();
        Self::new(data, len)
    }

    // iterators

    pub fn iter_as_dst(&mut self) -> SliceIterDst<'a, T> {
        SliceIterDst::from(self)
    }
}
