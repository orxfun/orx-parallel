use super::iter_ptr_dst::IterPtrDst;
use alloc::vec::Vec;
use core::marker::PhantomData;

pub trait IntoIterPtrDst<'a, T: 'a> {
    type Iter: IterPtrDst<'a, T>;

    fn into_iter_ptr_dst(self) -> Self::Iter;
}

// vec

pub struct SliceIterPtrDst<'a, T: 'a> {
    data: *mut T,
    exclusive_end: *mut T,
    p: PhantomData<&'a ()>,
}

impl<'a, T: 'a> IterPtrDst<'a, T> for SliceIterPtrDst<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }

    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> *mut T {
        debug_assert!(self.len() > 0);
        let value = self.data;
        self.data = unsafe { self.data.add(1) };
        value
    }
}

impl<'a, T: 'a> IntoIterPtrDst<'a, T> for &'a Vec<T> {
    type Iter = SliceIterPtrDst<'a, T>;

    fn into_iter_ptr_dst(self) -> Self::Iter {
        let n = self.len();
        let data = self.as_ptr() as *mut T;
        let exclusive_end = unsafe { data.add(n) };
        let p = PhantomData;
        SliceIterPtrDst {
            data,
            exclusive_end,
            p,
        }
    }
}
