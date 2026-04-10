use crate::results::ValIdx;
use core::marker::PhantomData;

/// Core structure for iterators over contiguous slices of data.
pub struct SliceIterPtr<'a, T: 'a> {
    data: *const ValIdx<T>,
    exclusive_end: *const ValIdx<T>,
    phantom: PhantomData<&'a ()>,
}

impl<T> Default for SliceIterPtr<'_, T> {
    fn default() -> Self {
        Self {
            data: core::ptr::null(),
            exclusive_end: core::ptr::null(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'a> SliceIterPtr<'a, T> {
    /// Creates a new iterator for `n` elements starting from the given `ptr`.
    ///
    /// # SAFETY
    ///
    /// - (i) either `ptr` is not-null or `n` is zero.
    pub unsafe fn new(ptr: *const ValIdx<T>, n: usize) -> Self {
        let exclusive_end = unsafe { ptr.add(n) };
        Self {
            data: ptr,
            exclusive_end,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.data == self.exclusive_end
    }

    #[inline(always)]
    fn remaining(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }
}
