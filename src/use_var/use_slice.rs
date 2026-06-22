use super::r#use::Use;
use core::marker::PhantomData;

/// Borrowed worker-local state backed by a mutable slice.
///
/// Each worker thread uses the element at its thread index.
/// This is typically used via `Par::use_slice`.
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

impl<'a, T: 'a> Use for UseSlice<'a, T> {
    type Item = T;

    #[inline]
    fn init_get(&self, thread_idx: usize) -> &mut Self::Item {
        assert!(
            thread_idx < self.len,
            "Out of bounds UseSlice access; slice has length {}, but access by {}-th thread.",
            self.len,
            thread_idx,
        );
        let ptr = unsafe { self.ptr.add(thread_idx) };
        unsafe { &mut *ptr }
    }

    #[inline]
    fn get(&mut self, thread_idx: usize) -> &mut Self::Item {
        self.init_get(thread_idx)
    }

    fn max_threads(&self) -> Option<usize> {
        Some(self.len)
    }
}
