use crate::algorithms::data_structures::slice_iter::SliceIterRef;
use core::marker::PhantomData;

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

// constructors

impl<'a, T> From<&'a [T]> for Slice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            data: value.as_ptr(),
            len: value.len(),
            phantom: PhantomData,
        }
    }
}

// methods

impl<'a, T> Slice<'a, T> {
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

    // iterators

    pub fn iter_over_ref(&self) -> SliceIterRef<'a, T> {
        SliceIterRef::from(self)
    }
}
