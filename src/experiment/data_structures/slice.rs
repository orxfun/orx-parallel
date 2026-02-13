use alloc::vec::Vec;
use core::ptr::slice_from_raw_parts;

/// # SAFETY
///
/// - (i) `data` is null and `len` is zero; or
/// - (ii) `data` is non-null and points to `len` consecutive initialized values.
#[inline(always)]
unsafe fn new_slice<'a, T>(data: *const T, len: usize) -> *const [T] {
    slice_from_raw_parts(data, len)
}

pub struct Slice<T>(*const [T]);

impl<T> From<&[T]> for Slice<T> {
    fn from(value: &[T]) -> Self {
        // SAFETY: constructing from a valid slice
        unsafe { Self::new(value.as_ptr(), value.len()) }
    }
}

impl<T> Slice<T> {
    /// # SAFETY
    ///
    /// - (i) `data` is null and `len` is zero; or
    /// - (ii) `data` is non-null and points to `len` consecutive initialized values.
    #[inline(always)]
    unsafe fn new(data: *const T, len: usize) -> Self {
        Self(slice_from_raw_parts(data, len))
    }

    pub fn empty() -> Self {
        // SAFETY: satisfies (i)
        unsafe { Self::new(core::ptr::null(), 0) }
    }

    pub fn from_vec_capacity(vec: &Vec<T>) -> Self {
        // SAFETY: constructing from a valid vec allocation.
        unsafe { Self::new(vec.as_ptr(), vec.capacity()) }
    }
}
