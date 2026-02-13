use crate::algorithms::data_structures::slice_iter::{SliceIterDst, SliceIterPtr, SliceIterRef};
use alloc::vec::Vec;
use core::{marker::PhantomData, ptr::slice_from_raw_parts};

/// A slice of contiguous data.
///
/// Its lifetime is bound to the owner of the data.
#[derive(Clone)]
pub struct Slice<'a, T> {
    data: *const T,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

/// SAFETY: [`Slice`] containing a raw pointer is allowed to send across threads.
/// Since the lifetime of the slice is bound to the owner of the data, this pointer
/// will be valid.
unsafe impl<T: Send> Send for Slice<'_, T> {}

/// SAFETY: [`Slice`] containing a raw pointer is allowed to shared across threads.
/// Since the lifetime of the slice is bound to the owner of the data, this pointer
/// will be valid.
unsafe impl<T: Sync> Sync for Slice<'_, T> {}

impl<'a, T> From<&'a [T]> for Slice<'a, T> {
    #[inline(always)]
    fn from(value: &'a [T]) -> Self {
        Self::new(value.as_ptr(), value.len())
    }
}

impl<'a, T> Slice<'a, T> {
    #[inline(always)]
    pub(super) fn new(data: *const T, len: usize) -> Self {
        Self {
            data,
            len,
            phantom: PhantomData,
        }
    }

    pub fn for_entire_capacity(value: &'a Vec<T>) -> Self {
        Self::new(value.as_ptr(), value.capacity())
    }

    pub fn clone(&self) -> Self {
        Self::new(self.data, self.len)
    }

    #[inline(always)]
    pub(super) fn data(&self) -> *const T {
        self.data
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&'a T> {
        match index < self.len {
            // # SAFETY: index is within the bounds and data is a valid pointer.
            true => Some(unsafe { &*self.data.add(index) }),
            false => None,
        }
    }

    pub fn last(&self) -> Option<&'a T> {
        match self.len {
            0 => None,
            n => Some(unsafe { &*self.data.add(n - 1) }),
        }
    }

    pub fn subslice_from(&self, begin: *const T) -> Self {
        debug_assert!(unsafe { begin.offset_from(self.data) >= 0 });
        debug_assert!(unsafe { self.data.add(self.len).offset_from(begin) > 0 });
        let len = self.len - unsafe { begin.offset_from(self.data) as usize };
        Self::new(begin, len)
    }

    pub fn as_slice(&self) -> &'a [T] {
        unsafe { &*slice_from_raw_parts(self.data, self.len) }
    }

    pub fn split_at(&self, at: usize) -> [Self; 2] {
        let left_len = at;
        let right_len = self.len - left_len;
        let left = Self::new(self.data, left_len);
        let right = Self::new(unsafe { self.data.add(left_len) }, right_len);
        [left, right]
    }

    pub fn split_at_mid(&self) -> [Self; 2] {
        self.split_at(self.len / 2)
    }

    pub fn copy_from_nonoverlapping(&self, source: &Self) {
        debug_assert_eq!(self.len, source.len);
        let dst = self.data as *mut T;
        unsafe { dst.copy_from_nonoverlapping(source.data, source.len) };
    }

    // iterators

    pub fn iter_over_ptr(&self) -> SliceIterPtr<'a, T> {
        SliceIterPtr::from(self)
    }

    pub fn iter_over_ref(&self) -> SliceIterRef<'a, T> {
        SliceIterRef::from(self)
    }

    pub fn iter_as_dst(&self) -> SliceIterDst<'a, T> {
        SliceIterDst::from(self)
    }
}
