use crate::experiment::data_structures::slice::Slice;
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

impl<'a, T: 'a> From<&Slice<T>> for SliceIterPtr<'a, T> {
    fn from(value: &Slice<T>) -> Self {
        match value.len() {
            0 => Self::default(),
            // SAFETY: `value.data() + n` marks the exclusive end which will never be read
            n => Self::new(value.data(), unsafe { value.data().add(n) }),
        }
    }
}

impl<'a, T: 'a> SliceIterPtr<'a, T> {
    fn new(data: *const T, exclusive_end: *const T) -> Self {
        Self {
            data,
            exclusive_end,
            phantom: PhantomData,
        }
    }

    /// Returns true if the end of the slice is reached.
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.data == self.exclusive_end
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
        let value = self.data;
        self.data = unsafe { self.data.add(1) };
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
    /// - (i) the element must be initialized.
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
}

// impl<'a, T: 'a> SliceIterPtr<'a, T> {
//     /// Returns true if the end of the slice is reached.
//     #[inline(always)]
//     pub fn is_finished(&self) -> bool {
//         self.data == self.exclusive_end
//     }

//     /// Returns a reference to the current value.
//     /// Returns None if the end of the slice is reached.
//     pub fn peek(&self) -> Option<*const T> {
//         match !self.is_finished() {
//             true => Some(self.data),
//             false => None,
//         }
//     }

//     #[inline(always)]
//     pub fn peek_unchecked(&self) -> *const T {
//         self.data
//     }

//     #[inline(always)]
//     pub fn current(&self) -> Option<&'a T> {
//         match !self.is_finished() {
//             true => Some(unsafe { &*self.data }),
//             false => None,
//         }
//     }

//     #[inline(always)]
//     pub unsafe fn current_unchecked(&self) -> &'a T {
//         debug_assert!(!self.is_finished());
//         unsafe { &*self.data }
//     }

//     /// Returns the next pointer.
//     ///
//     /// # SAFETY
//     ///
//     /// Does not perform bounds-check. Dereferencing the pointer that is
//     /// obtained by calling this method after the end of the slice is reached
//     /// leads to UB.
//     #[inline(always)]
//     pub unsafe fn next_unchecked(&mut self) -> *const T {
//         let value = self.data;
//         self.data = unsafe { self.data.add(1) };
//         value
//     }

//     pub fn next_if_leq<F>(&mut self, is_leq: F, pivot: &T) -> Option<*const T>
//     where
//         F: Fn(&T, &T) -> bool,
//     {
//         match self.current() {
//             Some(x) if is_leq(x, pivot) => {
//                 let value = Some(self.data);
//                 self.data = unsafe { self.data.add(1) };
//                 value
//             }
//             _ => None,
//         }
//     }

//     pub unsafe fn jump_to(&mut self, last_consumed: *const T) {
//         debug_assert!({ unsafe { last_consumed.offset_from(self.data) >= 0 } });
//         debug_assert!({ unsafe { self.exclusive_end.offset_from(self.data) > 0 } });
//         self.data = unsafe { last_consumed.add(1) };
//     }

//     /// Returns the remaining number of elements on the slice to be
//     /// iterated.
//     #[inline(always)]
//     fn remaining(&self) -> usize {
//         unsafe { self.exclusive_end.offset_from(self.data) as usize }
//     }

//     pub fn remaining_into_slice(&mut self) -> Slice<'a, T> {
//         let n = self.len();
//         let slice = Slice::new(self.data, n);
//         self.data = unsafe { self.data.add(n) };
//         slice
//     }
// }

// impl<'a, T: 'a> Iterator for SliceIterPtr<'a, T> {
//     type Item = *const T;

//     /// Returns the next pointer that is guaranteed to be valid.
//     /// Returns None if the end of the slice is reached.
//     #[inline(always)]
//     fn next(&mut self) -> Option<Self::Item> {
//         match !self.is_finished() {
//             true => Some(unsafe { self.next_unchecked() }),
//             false => None,
//         }
//     }

//     #[inline(always)]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.remaining(), Some(self.remaining()))
//     }
// }

// impl<'a, T: 'a> ExactSizeIterator for SliceIterPtr<'a, T> {
//     #[inline(always)]
//     fn len(&self) -> usize {
//         self.remaining()
//     }
// }
