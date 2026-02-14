use crate::experiment::data_structures::{
    slice::Slice, slice_iter_ptr::SliceIterPtr, slice_iter_ref::SliceIterRef,
};

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
    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns the number of remaining positions.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
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

    /// Returns the current pointer and progresses by `count` elements.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator must have at least `count` more elements; i.e.,
    ///   `self.remaining() >= count`.
    pub unsafe fn next_n_unchecked(&mut self, count: usize) -> *const T {
        unsafe { self.0.next_n_unchecked(count) }
    }

    /// Returns the current pointer and progresses the iterator to the next;
    /// returns None if the iterator `is_finished`.
    #[inline(always)]
    pub fn next(&mut self) -> Option<*const T> {
        self.0.next()
    }

    /// Does nothing if the iterator `is_finished` or `is_leq(current, pivot)`
    /// returns false.
    ///
    /// Otherwise, returns the current pointer and progresses the iterator to the next.
    #[inline(always)]
    pub fn next_if_leq<F>(&mut self, is_leq: F, pivot: &T) -> Option<*const T>
    where
        F: Fn(&T, &T) -> bool,
    {
        // SAFETY: SliceIterPtrSrc contains only initialized values
        unsafe { self.0.next_if_leq(is_leq, pivot) }
    }

    /// Brings the iterator to the end, skipping the remaining positions.
    pub(super) fn jump_to_end(&mut self) {
        self.0.jump_to_end();
    }

    /// Creates an iterator over references to values of the remaining elements
    /// of this iterator.
    pub fn values(&self) -> SliceIterRef<'a, T> {
        let ptr = unsafe { self.current_unchecked() };
        let n = self.len();
        unsafe { SliceIterRef::new(ptr, n) }
    }
}
