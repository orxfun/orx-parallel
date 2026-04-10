use super::slice_iter_ptr_src::SliceIterPtrSrc;

pub trait IterPtrDst<'a, T: 'a> {
    fn len(&self) -> usize;

    unsafe fn next_unchecked(&mut self) -> *mut T;

    /// # SAFETY
    ///
    /// (i) Both `src` and `self` must have at least 1 element.
    #[inline]
    unsafe fn write_one_from(&mut self, src: &mut SliceIterPtrSrc<'a, T>) {
        debug_assert!(self.len() > 0 && !src.is_finished());

        // SAFETY: satisfied by (i)
        let src = unsafe { src.next_unchecked() };

        // SAFETY: satisfied by (i)
        let dst = unsafe { self.next_unchecked() } as *mut T;

        unsafe { dst.copy_from_nonoverlapping(src, 1) };
    }

    /// # SAFETY
    ///
    /// (i) `src` and `self` must have equal lengths.
    unsafe fn write_rest_from(&mut self, src: SliceIterPtrSrc<'a, T>) {
        debug_assert_eq!(self.len(), src.len());

        for src_ptr in src {
            // SAFETY: having same lengths by (i), self cannot be finished
            let dst_ptr = unsafe { self.next_unchecked() };

            unsafe { dst_ptr.copy_from_nonoverlapping(src_ptr, 1) };
        }

        debug_assert_eq!(self.len(), 0);
    }
}
