use super::slice_src::SliceSrc;
use crate::results::ValIdx;
use core::marker::PhantomData;

/// Iterator over a slice of data that will be completely copied to another slice
/// before the iterator is consumed.
///
/// # SAFETY
///
/// While constructing this iterator, we must guarantee that all elements of it
/// are initialized since it will be used as source of values.
pub struct SliceIterPtrSrc<'a, T: 'a> {
    data: *const ValIdx<T>,
    exclusive_end: *const ValIdx<T>,
    phantom: PhantomData<&'a ()>,
}

impl<T> Default for SliceIterPtrSrc<'_, T> {
    fn default() -> Self {
        Self {
            data: core::ptr::null(),
            exclusive_end: core::ptr::null(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'a> SliceIterPtrSrc<'a, T> {
    pub fn new(slice: SliceSrc<'a, T>) -> Self {
        let raw = slice.destruct();
        let n = raw.len();
        let ptr = raw as *const ValIdx<T>;
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

    #[inline(always)]
    pub fn current_idx(&self) -> Option<usize> {
        todo!()
    }
}
