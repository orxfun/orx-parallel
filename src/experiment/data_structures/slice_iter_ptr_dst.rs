use crate::experiment::data_structures::{
    slice::Slice, slice_iter_ptr::SliceIterPtr, slice_iter_ptr_src::SliceIterPtrSrc,
};

/// Iterator over a slice of data that will be completely filled with values
/// before the iterator is consumed.
///
/// The slice might initially be uninitialized, but will completely be initialized.
pub struct SliceIterPtrDst<'a, T: 'a>(SliceIterPtr<'a, T>);

impl<T> Default for SliceIterPtrDst<'_, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'a, T: 'a> From<&Slice<T>> for SliceIterPtrDst<'a, T> {
    fn from(value: &Slice<T>) -> Self {
        Self(value.into())
    }
}

impl<'a, T: 'a> SliceIterPtrDst<'a, T> {
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

    /// Pulls the next element from `src`, writes it to next position of `self`.
    ///
    /// Progresses both `self` and `src` by one element.
    ///
    /// # SAFETY
    ///
    /// - (i) `src` cannot be `is_finished`
    /// - (ii) `self` cannot be `is_finished`
    /// - (iii) `src` and `self` cannot be overlapping
    #[inline(always)]
    pub unsafe fn write_one_from(&mut self, src: &mut SliceIterPtrSrc<'a, T>) {
        debug_assert!(!self.is_finished() && !src.is_finished());

        // SAFETY: satisfied by (i)
        let src = unsafe { src.next_unchecked() };

        // SAFETY: satisfied by (ii)
        let dst = unsafe { self.0.next_unchecked() } as *mut T;

        // SAFETY: satisfied by (iii)
        unsafe { dst.copy_from_nonoverlapping(src, 1) };
    }

    pub unsafe fn write_many_from(&mut self, src: &mut SliceIterPtrSrc<'a, T>, count: usize) {
        debug_assert!(!self.is_finished() && !src.is_finished());

        // SAFETY: satisfied by (i)
        let src = unsafe { src.next_unchecked() };

        // SAFETY: satisfied by (ii)
        let dst = unsafe { self.0.next_unchecked() } as *mut T;

        // SAFETY: satisfied by (iii)
        unsafe { dst.copy_from_nonoverlapping(src, 1) };

        //         #[inline(always)]
        // pub unsafe fn write_many_unchecked(
        //     &mut self,
        //     src_begin: *const T,
        //     src_end_inclusive: *const T,
        // ) {
        //     let count = unsafe { src_end_inclusive.offset_from(src_begin) + 1 } as usize;
        //     debug_assert!(self.0.len() >= count);
        //     let dst = unsafe { self.0.next_many_unchecked(count) };
        //     unsafe { dst.copy_from_nonoverlapping(src_begin, count) };
        // }
    }

    /// Pulls all remaining elements from `src`, writes them to remaining
    /// positions of `self`.
    ///
    /// Progresses both `self` and `src` to the end.
    ///
    /// # SAFETY
    ///
    /// - (i) `src` and `self` mut have equal number of remaining elements
    /// - (ii) `src` and `self` cannot be overlapping
    pub unsafe fn write_rest_from(&mut self, src: &mut SliceIterPtrSrc<'a, T>) {
        debug_assert!(!self.is_finished() && !src.is_finished());
        debug_assert_eq!(self.len(), src.len());

        if let Some(src) = src.next() {
            // SAFETY: having same lengths with src by (i), self cannot be finished
            let dst = unsafe { self.0.next_unchecked() } as *mut T;

            // SAFETY: satisfied by (ii)
            unsafe { dst.copy_from_nonoverlapping(src, self.len()) };
        }

        self.0.jump_to_end();
        src.jump_to_end();
    }

    #[inline(always)]
    pub unsafe fn write_one_unchecked(&mut self, src: *const T) {
        // debug_assert!(self.0.len() > 0);
        // let dst = unsafe { self.0.next_unchecked() };
        // unsafe { dst.copy_from_nonoverlapping(src, 1) };
    }

    #[inline(always)]
    pub unsafe fn write_many_unchecked(
        &mut self,
        src_begin: *const T,
        src_end_inclusive: *const T,
    ) {
        // let count = unsafe { src_end_inclusive.offset_from(src_begin) + 1 } as usize;
        // debug_assert!(self.0.len() >= count);
        // let dst = unsafe { self.0.next_many_unchecked(count) };
        // unsafe { dst.copy_from_nonoverlapping(src_begin, count) };
    }

    #[inline(always)]
    pub unsafe fn write_remaining_from(&mut self, src: &Slice<T>) {
        // debug_assert_eq!(src.len(), self.0.len());

        // match self.0.next_remaining() {
        //     Some((dst, len)) => {
        //         debug_assert_eq!(src.len(), len);
        //         unsafe { dst.copy_from_nonoverlapping(src.data(), len) };
        //     }
        //     None => {}
        // }
        // debug_assert!(self.0.is_finished());
    }
}
