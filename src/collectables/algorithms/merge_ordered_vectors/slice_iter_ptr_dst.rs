use super::slice_iter_ptr_src::SliceIterPtrSrc;

pub trait SliceIterPtrDst<'a, T: 'a> {
    fn len(&self) -> usize;

    unsafe fn next_unchecked(&mut self) -> *mut T;

    unsafe fn write_rest_from(&mut self, src: SliceIterPtrSrc<'a, T>) {
        debug_assert_eq!(self.len(), src.len());

        for src_ptr in src {
            // SAFETY: having same lengths with src, self cannot be finished
            let dst_ptr = unsafe { self.next_unchecked() };

            unsafe { dst_ptr.copy_from_nonoverlapping(src_ptr, 1) };
        }

        debug_assert_eq!(self.len(), 0);
    }
}
