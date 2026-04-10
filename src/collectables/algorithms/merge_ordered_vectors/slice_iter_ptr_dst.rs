use super::slice_iter_ptr_src::SliceIterPtrSrc;

pub trait SliceIterPtrDst<'a, T: 'a> {
    fn len(&self) -> usize;

    unsafe fn write_rest_from(&mut self, src: &mut SliceIterPtrSrc<'a, T>) {
        if let Some(src) = src.next() {
            let count = self.len();

            // SAFETY: having same lengths with src by (i), self cannot be finished
            let dst = unsafe { self.0.next_unchecked() } as *mut T;

            // SAFETY: satisfied by (ii)
            unsafe { dst.copy_from_nonoverlapping(src, count) };
        }

        self.0.jump_to_end();
        src.jump_to_end();
    }
}
