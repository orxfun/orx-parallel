use crate::experiment::data_structures::{slice::Slice, slice_iter_ptr::SliceIterPtr};
use core::marker::PhantomData;

/// Iterator over a slice of data that will be completely copied to another slice
/// before the iterator is consumed.
pub struct SliceIterPtrSrc<'a, T: 'a>(SliceIterPtr<'a, T>);

impl<T> Default for SliceIterPtrSrc<'_, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'a, T: 'a> From<&Slice<T>> for SliceIterPtrSrc<'a, T> {
    fn from(value: &Slice<T>) -> Self {
        Self(value.into())
    }
}

impl<'a, T: 'a> SliceIterPtrSrc<'a, T> {
    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns a reference to the current element.
    /// Returns None if the iterator `is_finished`.
    ///
    /// # SAFETY
    ///
    /// - (i) the element must be initialized.
    /// - (ii) the iterator cannot be `is_finished`; otherwise, we
    ///   will have an UB due to dereferencing an invalid pointer.
    #[inline(always)]
    pub unsafe fn current_unchecked(&self) -> &'a T {
        unsafe { self.0.current_unchecked() }
    }

    /// Returns the current pointer and progresses the iterator to the next.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator cannot be `is_finished`; otherwise, the
    ///   obtained pointer does not belong to the slice the iterator
    ///   is created for.
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> *const T {
        unsafe { self.0.next_unchecked() }
    }
}
