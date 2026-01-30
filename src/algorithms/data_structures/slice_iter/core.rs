use core::marker::PhantomData;

/// Core structure for iterators over contiguous slices of data.
pub struct SliceIterCore<'a, T: 'a> {
    data: *const T,
    exclusive_end: *const T,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T: 'a> SliceIterCore<'a, T> {
    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.data == self.exclusive_end
    }

    /// Returns a reference to the current value.
    /// Returns None if the end of the slice is reached.
    pub fn peek(&self) -> Option<*const T> {
        match !self.is_finished() {
            true => Some(self.data),
            false => None,
        }
    }

    /// Returns the next pointer.
    ///
    /// # SAFETY
    ///
    /// Does not perform bounds-check. Dereferencing the pointer that is
    /// obtained by calling this method after the end of the slice is reached
    /// leads to UB.
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> *const T {
        let value = self.data;
        self.data = unsafe { self.data.add(1) };
        value
    }

    /// Returns the next pointer that is guaranteed to be valid.
    /// Returns None if the end of the slice is reached.
    #[inline(always)]
    pub fn next(&mut self) -> Option<*const T> {
        match !self.is_finished() {
            true => Some(unsafe { self.next_unchecked() }),
            false => None,
        }
    }

    /// Returns the remaining number of elements on the slice to be
    /// iterated.
    #[inline(always)]
    pub fn len(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }
}
