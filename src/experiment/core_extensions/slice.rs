use alloc::vec::Vec;
use core::ptr::slice_from_raw_parts;

/// Extension methods for slices.
pub trait SliceExtensions<'a, T> {
    fn from_vec_capacity(vec: &'a Vec<T>) -> Self;
}

impl<'a, T> SliceExtensions<'a, T> for &'a [T] {
    fn from_vec_capacity(vec: &'a Vec<T>) -> Self {
        unsafe { &*slice_from_raw_parts(vec.as_ptr(), vec.capacity()) }
    }
}
