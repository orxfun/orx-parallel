use crate::experiment::data_structures::{slice_iter_ptr::SliceIterPtr, slice_src::SliceSrc};
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
    /// # SAFETY
    ///
    /// Since `slice: SliceSrc` satisfies that all elements of it are initialized,
    /// we satisfy the construction condition for this iterator.
    pub fn new(slice: SliceSrc<'a, T>) -> Self {
        let raw = slice.destruct();
        // SAFETY: requirement satisfied by `SliceSrc`
        Self(unsafe { SliceIterPtr::new(raw as *const T, raw.len()) })
    }

    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    /// Returns a reference to the current element.
    /// Returns None if the iterator `is_finished`.
    #[inline(always)]
    pub fn current(&self) -> Option<&'a T> {
        // SAFETY: all elements are initialized
        unsafe { self.0.current() }
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
        // SAFETY: matching req't and cond'n (i)
        unsafe { self.0.next_unchecked() }
    }

    /// Returns the current pointer and progresses by `count` elements.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator must have at least `count` more elements; i.e.,
    ///   `self.remaining() >= count`.
    pub unsafe fn next_n_unchecked(&mut self, count: usize) -> *const T {
        // SAFETY: matching req't and cond'n (i)
        unsafe { self.0.next_n_unchecked(count) }
    }

    /// Brings the iterator to the end, skipping the remaining positions.
    pub(super) fn jump_to_end(&mut self) {
        self.0.jump_to_end();
    }

    /// Creates an iterator over references to values of the remaining elements
    /// of this iterator.
    pub fn values(&self) -> core::slice::Iter<'a, T> {
        self.as_slice().iter()
    }

    /// Creates a slice over references to values of the remaining elements
    /// of this iterator.
    pub fn as_slice(&self) -> &'a [T] {
        let ptr = self.0.peek();
        let n = self.len();
        // SAFETY: SliceIterPtrSrc guarantees initialized values
        unsafe { from_raw_parts(ptr, n) }
    }
}

impl<'a, T> Iterator for SliceIterPtrSrc<'a, T> {
    type Item = *const T;

    /// Returns the current pointer and progresses the iterator to the next;
    /// returns None if the iterator `is_finished`.
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len();
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for SliceIterPtrSrc<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
}
