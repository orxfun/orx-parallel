use crate::experiment::data_structures::{
    slice::Slice, slice_iter_ptr::SliceIterPtr, slice_iter_ptr_src::SliceIterPtrSrc,
};
use core::marker::PhantomData;

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

    /// Pulls one element from `src` writes it to `self` as destination.
    ///
    /// Progresses both `self` and `src` by one element.
    ///
    /// # SAFETY
    ///
    /// - (i) `src` cannot be `is_finished`
    /// - (ii) `self` cannot be `is_finished`
    /// - (iii) `src` and `self` cannot be overlapping
    #[inline(always)]
    pub unsafe fn write_one(&mut self, src: &mut SliceIterPtrSrc<'a, T>) {
        // SAFETY: satisfied by (i)
        let src = unsafe { src.next_unchecked() };

        // SAFETY: satisfied by (ii)
        let dst = unsafe { self.0.next_unchecked() } as *mut T;

        // SAFETY: satisfied by (iii)
        unsafe { dst.copy_from_nonoverlapping(src, 1) };
    }

    pub unsafe fn write_remaining(&mut self, src: &mut SliceIterPtrSrc<'a, T>) {
        //
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
