use crate::algorithms::data_structures::slice_iter::{SliceIterPtr, SliceIterRef};
use core::{marker::PhantomData, ptr::slice_from_raw_parts};

/// A slice of contiguous data.
///
/// Its lifetime is bound to the owner of the data.
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

    pub fn subslice_from(&self, begin: *const T) -> Self {
        debug_assert!(unsafe { begin.offset_from(self.data) >= 0 });
        debug_assert!(unsafe { self.data.add(self.len).offset_from(begin) > 0 });
        let len = self.len - unsafe { begin.offset_from(self.data) as usize };
        Self::new(begin, len)
    }

    pub fn as_slice(&self) -> &'a [T] {
        unsafe { &*slice_from_raw_parts(self.data, self.len) }
    }

    // iterators

    pub fn iter_over_ptr(&self) -> SliceIterPtr<'a, T> {
        SliceIterPtr::from(self)
    }

    pub fn iter_over_ref(&self) -> SliceIterRef<'a, T> {
        SliceIterRef::from(self)
    }
}
