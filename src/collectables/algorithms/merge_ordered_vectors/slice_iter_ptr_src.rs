use super::slice_iter_ptr::SliceIterPtr;
use super::slice_src::SliceSrc;
use crate::results::ValIdx;
use core::slice::from_raw_parts;

/// Iterator over a slice of data that will be completely copied to another slice
/// before the iterator is consumed.
///
/// # SAFETY
///
/// While constructing this iterator, we must guarantee that all elements of it
/// are initialized since it will be used as source of values.
pub struct SliceIterPtrSrc<'a, T: 'a>(SliceIterPtr<'a, T>);

impl<T> Default for SliceIterPtrSrc<'_, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'a, T: 'a> SliceIterPtrSrc<'a, T> {
    pub fn new(slice: SliceSrc<'a, T>) -> Self {
        let raw = slice.destruct();
        // SAFETY: requirement satisfied by `SliceSrc`
        Self(unsafe { SliceIterPtr::new(raw as *const ValIdx<T>, raw.len()) })
    }

    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }
}
