use super::over_mut_ptr::SliceIterMutPtr;
use crate::algorithms::data_structures::slice::Slice;

pub struct SliceIterDst<'a, T: 'a>(SliceIterMutPtr<'a, T>);

impl<'a, T: 'a> From<&mut Slice<'a, T>> for SliceIterDst<'a, T> {
    fn from(value: &mut Slice<'a, T>) -> Self {
        Self(value.into())
    }
}

impl<'a, T: 'a> SliceIterDst<'a, T> {
    #[inline(always)]
    pub unsafe fn write_one_unchecked(&mut self, src: *const T) {
        debug_assert!(self.0.len() > 0);
        let dst = unsafe { self.0.next_unchecked() };
        unsafe { dst.copy_from_nonoverlapping(src, 1) };
    }

    #[inline(always)]
    pub unsafe fn write_remaining_from(&mut self, src: &Slice<'a, T>) {
        debug_assert_eq!(src.len(), self.0.len());

        match self.0.next_remaining() {
            Some((dst, len)) => {
                debug_assert_eq!(src.len(), len);
                unsafe { dst.copy_from_nonoverlapping(src.data(), len) };
            }
            None => {}
        }
        debug_assert!(self.0.is_finished());
    }
}
