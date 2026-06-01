use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;

pub trait Use {
    type Item;

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item;
}

pub struct UseVec<T: Send, F: Fn(usize) -> T + Sync> {
    init: F,
    cache: ConcurrentOrderedBag<T>,
}

impl<T: Send, F: Fn(usize) -> T + Sync> Use for UseVec<T, F> {
    type Item = T;

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
        let use_var = (self.init)(thread_idx);
        unsafe { self.cache.set_value(thread_idx, use_var) };
        unsafe { &mut *self.cache.ptr_mut(thread_idx) }
    }
}

pub struct UseSlice<'a, T: 'a> {
    ptr: *mut T,
    len: usize,
    p: PhantomData<fn() -> &'a ()>,
}

unsafe impl<'a, T: 'a> Sync for UseSlice<'a, T> {}

impl<'a, T: 'a> Use for UseSlice<'a, T> {
    type Item = T;

    fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
        let ptr = unsafe { self.ptr.add(thread_idx) };
        unsafe { &mut *ptr }
    }
}

// VV

pub struct UseVV<T1, F1, T2, F2>
where
    T1: Send,
    T2: Send,
    F1: Fn(usize) -> T1 + Sync,
    F2: Fn(usize) -> T2 + Sync,
{
    a: UseVec<T1, F1>,
    b: UseVec<T2, F2>,
    cache: ConcurrentOrderedBag<(*mut T1, *mut T2)>,
}

// impl<T1, F1, T2, F2> Use for UseVV<T1, F1, T2, F2>
// where
//     T1: Send,
//     T2: Send,
//     F1: Fn(usize) -> T1 + Sync,
//     F2: Fn(usize) -> T2 + Sync,
// {
//     type Item = (*mut T1, *mut T2);

//     fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
//         let a = self.a.init_get(thread_idx) as *mut T1;
//         let b = self.b.init_get(thread_idx) as *mut T2;
//         unsafe { self.cache.set_value(thread_idx, use_var) };

//         todo!()
//     }
// }
