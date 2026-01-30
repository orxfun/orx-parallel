use super::over_ptr::SliceIterPtr;
use crate::algorithms::data_structures::slice::Slice;

pub struct SliceIterDst<'a, T: 'a>(SliceIterPtr<'a, T>);

impl<'a, T: 'a> From<&Slice<'a, T>> for SliceIterDst<'a, T> {
    fn from(value: &Slice<'a, T>) -> Self {
        Self(value.into())
    }
}

impl<'a, T: 'a> SliceIterDst<'a, T> {
    // #[inline(always)]
    // pub fn write_one(&mut self, src: *const T) {
    //     debug_assert!(self.0.len() > 0);
    //     unsafe { self.data.copy_from_nonoverlapping(src, 1) };
    //     self.data = unsafe { self.data.add(1) };
    // }
}
