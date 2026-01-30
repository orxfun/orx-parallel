use crate::algorithms::data_structures::slice::Slice;
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

impl<'a, T: 'a> From<&Slice<'a, T>> for SliceIterPtr<'a, T> {
    fn from(value: &Slice<'a, T>) -> Self {
        match value.len() {
            0 => Self::default(),
            n => Self {
                data: value.data(),
                exclusive_end: unsafe { value.data().add(n) },
                phantom: PhantomData,
            },
        }
    }
}

impl<'a, T: 'a> SliceIterPtr<'a, T> {
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

    /// Returns the remaining number of elements on the slice to be
    /// iterated.
    #[inline(always)]
    fn remaining(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }
}

impl<'a, T: 'a> Iterator for SliceIterPtr<'a, T> {
    type Item = *const T;

    /// Returns the next pointer that is guaranteed to be valid.
    /// Returns None if the end of the slice is reached.
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
