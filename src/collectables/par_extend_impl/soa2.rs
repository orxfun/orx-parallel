use crate::impl_par_extend_for_soa;
use alloc::{vec, vec::Vec};
use core::iter::Zip;

pub struct Soa2<T1, T2> {
    v1: Vec<T1>,
    v2: Vec<T2>,
}

impl<T1, T2> Soa2<T1, T2> {
    pub fn new() -> Self {
        Self {
            v1: Vec::new(),
            v2: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            v1: Vec::with_capacity(capacity),
            v2: Vec::with_capacity(capacity),
        }
    }

    pub fn into_inner(self) -> (Vec<T1>, Vec<T2>) {
        (self.v1, self.v2)
    }

    #[inline(always)]
    pub fn push(&mut self, (i1, i2): (T1, T2)) {
        self.v1.push(i1);
        self.v2.push(i2);
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.v1.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.v1.is_empty()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.v1.reserve(additional);
        self.v2.reserve(additional);
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        unsafe { self.v1.set_len(new_len) };
        unsafe { self.v2.set_len(new_len) };
    }

    pub fn as_ptr(&self) -> (*const T1, *const T2) {
        (self.v1.as_ptr(), self.v2.as_ptr())
    }

    pub fn as_mut_ptr(&mut self) -> (*mut T1, *mut T2) {
        (self.v1.as_mut_ptr(), self.v2.as_mut_ptr())
    }
}

impl<T1, T2> Default for Soa2<T1, T2> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T1, T2> Extend<(T1, T2)> for Soa2<T1, T2> {
    fn extend<I: IntoIterator<Item = (T1, T2)>>(&mut self, iter: I) {
        for (i1, i2) in iter {
            self.v1.push(i1);
            self.v2.push(i2);
        }
    }
}

impl<T1, T2> IntoIterator for Soa2<T1, T2> {
    type Item = (T1, T2);

    type IntoIter = Zip<vec::IntoIter<T1>, vec::IntoIter<T2>>;

    fn into_iter(self) -> Self::IntoIter {
        self.v1.into_iter().zip(self.v2)
    }
}

impl_par_extend_for_soa!(Soa2, 2);
