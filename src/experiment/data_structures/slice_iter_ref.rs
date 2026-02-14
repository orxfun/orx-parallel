use crate::experiment::data_structures::slice_iter_ptr::SliceIterPtr;
use alloc::vec::Vec;

/// Iterator over references of values over a contiguous slice.
///
/// # SAFETY
///
/// While constructing this iterator, we must guarantee that all elements of the
/// iterator is initialized since it will be used as source of values.
pub struct SliceIterRef<'a, T: 'a>(SliceIterPtr<'a, T>);

impl<'a, T: 'a> SliceIterRef<'a, T> {
    pub unsafe fn new(ptr: *const T, n: usize) -> Self {
        Self(unsafe { SliceIterPtr::new(ptr, n) })
    }
}

impl<'a, T: 'a> Iterator for SliceIterRef<'a, T> {
    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: initialized values are guaranteed on construction.
        self.0.next().map(|ptr| unsafe { &*ptr })
    }
}
