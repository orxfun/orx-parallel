use super::over_mut_ptr::SliceIterMutPtr;
use crate::algorithms::data_structures::{Slice, slice_mut::SliceMut};

pub struct SliceIterDst<'a, T: 'a>(SliceIterMutPtr<'a, T>);

impl<'a, T: 'a> From<&mut SliceMut<'a, T>> for SliceIterDst<'a, T> {
    fn from(value: &mut SliceMut<'a, T>) -> Self {
        Self(value.into())
    }
}

impl<'a, T: 'a> From<&Slice<'a, T>> for SliceIterDst<'a, T> {
    fn from(value: &Slice<'a, T>) -> Self {
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
    pub unsafe fn write_many_unchecked(
        &mut self,
        src_begin: *const T,
        src_end_inclusive: *const T,
    ) {
        let count = unsafe { src_end_inclusive.offset_from(src_begin) + 1 } as usize;
        debug_assert!(self.0.len() >= count);
        let dst = unsafe { self.0.next_many_unchecked(count) };
        unsafe { dst.copy_from_nonoverlapping(src_begin, count) };
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
