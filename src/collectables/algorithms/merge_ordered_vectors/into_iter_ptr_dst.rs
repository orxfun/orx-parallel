use super::iter_ptr_dst::IterPtrDst;
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::SplitVec;

pub trait IntoIterPtrDst<'a, T: 'a> {
    type Iter: IterPtrDst<'a, T>;

    fn into_iter_ptr_dst(self) -> Self::Iter;
}

// Vec

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

impl<'a, T: 'a> IntoIterPtrDst<'a, T> for &'a [T] {
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

// FixedVec

impl<'a, T: 'a> IntoIterPtrDst<'a, T> for &'a FixedVec<T> {
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

// Vec2

struct Ptrs<'a, T: 'a> {
    begin: *const Vec<T>,
    num_fragments: usize,
    p: PhantomData<&'a ()>,
}

struct Ptr<'a, T: 'a> {
    current: *const T,
    stopper: *const T,
    p: PhantomData<&'a ()>,
}

pub struct IterPtr<'a, T: 'a> {
    ptrs: Ptrs<'a, T>,
    current_f: usize,
    current: Ptr<'a, T>,
    p: PhantomData<&'a ()>,
}

impl<'a, T: 'a> IterPtrDst<'a, T> for IterPtr<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        // unsafe { self.exclusive_end.offset_from(self.data) as usize }
        todo!()
    }

    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> *mut T {
        // debug_assert!(self.len() > 0);
        // let value = self.data;
        // self.data = unsafe { self.data.add(1) };
        // value
        todo!()
    }
}

pub struct WrapIterPtrDst<T, I>(I)
where
    I: Iterator<Item = *const T>;

impl<'a, T: 'a> IntoIterPtrDst<'a, T> for &'a SplitVec<T> {
    type Iter = SliceIterPtrDst<'a, T>;

    fn into_iter_ptr_dst(self) -> Self::Iter {
        let x = self.fragments();
        let x = unsafe { self.iter_ptr() };
        todo!()
        // let n = self.len();
        // let data = self.as_ptr() as *mut T;
        // let exclusive_end = unsafe { data.add(n) };
        // let p = PhantomData;
        // SliceIterPtrDst {
        //     data,
        //     exclusive_end,
        //     p,
        // }
    }
}
