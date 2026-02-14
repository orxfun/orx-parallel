use core::marker::PhantomData;

/// Core structure for iterators over contiguous slices of data.
pub struct SliceIterPtr<'a, T: 'a> {
    data: *const T,
    exclusive_end: *const T,
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
    pub unsafe fn new(ptr: *const T, n: usize) -> Self {
        let exclusive_end = unsafe { ptr.add(n) };
        Self {
            data: ptr,
            exclusive_end,
            phantom: PhantomData,
        }
    }

    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.data == self.exclusive_end
    }

    /// Returns the remaining number of elements on the slice to be
    /// iterated.
    #[inline(always)]
    fn remaining(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }

    // non-progressing methods

    /// Returns the current pointer.
    /// Returns None if the iterator `is_finished`.
    pub fn peek(&self) -> Option<*const T> {
        match !self.is_finished() {
            true => Some(self.data),
            false => None,
        }
    }

    /// Returns the current pointer.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator cannot be `is_finished`; otherwise, the
    ///   obtained pointer does not belong to the slice the iterator
    ///   is created for.
    #[inline(always)]
    pub unsafe fn peek_unchecked(&self) -> *const T {
        debug_assert!(!self.is_finished());
        self.data
    }

    /// Returns a reference to the current element.
    /// Returns None if the iterator `is_finished`.
    ///
    /// # SAFETY
    ///
    /// Bounds check is applied. However, the following safety
    /// requirement must be satisfied.
    ///
    /// - (i) the element must be initialized.
    #[inline(always)]
    pub unsafe fn current(&self) -> Option<&'a T> {
        match !self.is_finished() {
            // SAFETY: the value is initialized.
            true => Some(unsafe { &*self.data }),
            false => None,
        }
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
        debug_assert!(!self.is_finished());
        unsafe { &*self.data }
    }

    // progressing methods

    /// Returns the current pointer and progresses the iterator to the next.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator cannot be `is_finished`; otherwise, the
    ///   obtained pointer does not belong to the slice the iterator
    ///   is created for.
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> *const T {
        debug_assert!(!self.is_finished());
        let value = self.data;
        self.data = unsafe { self.data.add(1) };
        value
    }

    /// Returns the current pointer and progresses by `count` elements.
    ///
    /// # SAFETY
    ///
    /// - (i) the iterator must have at least `count` more elements; i.e.,
    ///   `self.remaining() >= count`.
    pub unsafe fn next_n_unchecked(&mut self, count: usize) -> *const T {
        debug_assert!(self.remaining() >= count);
        let value = self.data;
        self.data = unsafe { self.data.add(count) };
        value
    }

    /// Does nothing if the iterator `is_finished` or `is_leq(current, pivot)`
    /// returns false.
    ///
    /// Otherwise, returns the current pointer and progresses the iterator to the next.
    ///
    /// # SAFETY
    ///
    /// Bounds check is applied. However, the following safety
    /// requirement must be satisfied.
    ///
    /// - (i) the elements of `self` must be initialized.
    pub unsafe fn next_if_leq<F>(&mut self, is_leq: F, pivot: &T) -> Option<*const T>
    where
        F: Fn(&T, &T) -> bool,
    {
        // SAFETY: satisfied by (i)
        match unsafe { self.current() } {
            Some(x) if is_leq(x, pivot) => {
                let value = Some(self.data);
                self.data = unsafe { self.data.add(1) };
                value
            }
            _ => None,
        }
    }

    /// Brings the iterator to the end, skipping the remaining positions.
    pub fn jump_to_end(&mut self) {
        self.data = self.exclusive_end
    }
}

impl<'a, T: 'a> Iterator for SliceIterPtr<'a, T> {
    type Item = *const T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match !self.is_finished() {
            true => Some(unsafe { self.next_unchecked() }),
            false => None,
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining(), Some(self.remaining()))
    }
}

impl<'a, T: 'a> ExactSizeIterator for SliceIterPtr<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}
