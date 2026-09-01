use crate::collectables::par_extend::{Contiguous, ParExtend};
use alloc::vec::Vec;

impl<T> ParExtend<T> for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn push_one(&mut self, value: T) {
        self.push(value);
    }
}

impl<T> Contiguous<T> for Vec<T> {
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }

    fn capacity(&self) -> usize {
        Vec::capacity(self)
    }

    unsafe fn ptr(&mut self, idx: usize) -> *mut T {
        debug_assert!(idx < self.len(), "index out of bounds");
        let p = self.as_mut_ptr();
        unsafe { p.add(idx) }
    }

    unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity(), "setting len beyond capacity");
        unsafe { Vec::set_len(self, new_len) };
    }
}
