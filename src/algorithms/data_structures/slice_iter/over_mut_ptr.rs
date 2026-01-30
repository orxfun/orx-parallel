use crate::algorithms::data_structures::slice::Slice;
use core::marker::PhantomData;

/// Core structure for iterators over contiguous slices of data.
pub struct SliceIterMutPtr<'a, T: 'a> {
    data: *mut T,
    exclusive_end: *mut T,
    phantom: PhantomData<&'a ()>,
}

impl<T> Default for SliceIterMutPtr<'_, T> {
    fn default() -> Self {
        Self {
            data: core::ptr::null_mut(),
            exclusive_end: core::ptr::null_mut(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'a> From<&mut Slice<'a, T>> for SliceIterMutPtr<'a, T> {
    fn from(value: &mut Slice<'a, T>) -> Self {
        match value.len() {
            0 => Self::default(),
            n => {
                let data = value.data() as *mut T;
                Self {
                    data: data,
                    exclusive_end: unsafe { data.add(n) },
                    phantom: PhantomData,
                }
            }
        }
    }
}

impl<'a, T: 'a> SliceIterMutPtr<'a, T> {
    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.data == self.exclusive_end
    }

    /// Returns the next pointer.
    ///
    /// # SAFETY
    ///
    /// Does not perform bounds-check. Dereferencing the pointer that is
    /// obtained by calling this method after the end of the slice is reached
    /// leads to UB.
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> *mut T {
        let value = self.data;
        self.data = unsafe { self.data.add(1) };
        value
    }

    /// Returns the tuple of the next pointer and the positive number of remaining elements.
    /// Note that the returned pointer is valid and the length is positive.
    ///
    /// If the iterator is already consumed, the method returns None.
    ///
    /// Therefore, it will never return an invalid pointer.
    pub fn next_remaining(&mut self) -> Option<(*mut T, usize)> {
        match self.len() {
            0 => None,
            n => {
                let value = Some((self.data, n));
                self.data = unsafe { self.data.add(n) };
                value
            }
        }
    }

    /// Returns the remaining number of elements on the slice to be
    /// iterated.
    #[inline(always)]
    fn remaining(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }
}

impl<'a, T: 'a> Iterator for SliceIterMutPtr<'a, T> {
    type Item = *mut T;

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

impl<'a, T: 'a> ExactSizeIterator for SliceIterMutPtr<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.remaining()
    }
}
