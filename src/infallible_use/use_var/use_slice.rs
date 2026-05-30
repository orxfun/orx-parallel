use crate::infallible_use::Using;
use core::marker::PhantomData;

pub struct UseSlice<'a, T: 'a> {
    ptr: *mut T,
    len: usize,
    p: PhantomData<fn() -> &'a ()>,
}

impl<'a, T: 'a> UseSlice<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        let ptr = slice.as_mut_ptr();
        let len = slice.len();
        let p = PhantomData;
        Self { ptr, len, p }
    }
}

unsafe impl<'a, T: 'a> Sync for UseSlice<'a, T> {}

impl<'a, T: 'a> Using for UseSlice<'a, T> {
    type Item = T;

    type ItemKind<'i>
        = &'a mut T
    where
        Self: 'i;

    fn create(&self, thread_idx: usize) -> Self::Item {
        todo!()
    }

    fn get(&self, thread_idx: usize) -> Self::ItemKind<'_> {
        assert!(
            thread_idx < self.len,
            "Out of bounds UseSlice access; slice has length {}, but access by {}-th thread.",
            self.len,
            thread_idx,
        );
        let ptr = unsafe { self.ptr.add(thread_idx) };
        unsafe { &mut *ptr }
    }
}
