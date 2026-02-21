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

    /// Returns the number of remaining positions.
    #[inline(always)]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a reference to the current element.
    /// Returns None if the iterator `is_finished`.
    #[inline(always)]
    pub fn current(&self) -> Option<&'a T> {
        // SAFETY: all elements are initialized
        unsafe { self.0.current() }
    }

    /// Returns a reference to the current element, without bounds check.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator cannot be `is_finished`; otherwise, we
    ///   will have an UB due to dereferencing an invalid pointer.
    #[inline(always)]
    pub unsafe fn current_unchecked(&self) -> &'a T {
        // SAFETY: matching req't and cond'n (i)
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

    /// Returns the current pointer and progresses the iterator to the next;
    /// returns None if the iterator `is_finished`.
    #[inline(always)]
    pub fn next(&mut self) -> Option<*const T> {
        self.0.next()
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
        unsafe { &*from_raw_parts(ptr, n) }
    }
}
