use super::iter_ptr_dst::IterPtrDst;
use crate::results::ValIdx;
use alloc::vec::Vec;
use core::marker::PhantomData;

pub trait IntoIterPtrDst<'a, T: 'a> {
    type Iter: IterPtrDst<'a, T>;

    fn into_iter_ptr_dst(self) -> Self::Iter;
}

// vec

pub struct SliceIterPtrDst<'a, T: 'a> {
    data: *const ValIdx<T>,
    exclusive_end: *const ValIdx<T>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T: 'a> IterPtrDst<'a, T> for SliceIterPtrDst<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }

    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> *mut T {
        todo!()
    }
}
