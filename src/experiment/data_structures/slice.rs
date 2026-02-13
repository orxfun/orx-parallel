use alloc::vec::Vec;
use core::ptr::slice_from_raw_parts;

/// A raw slice of contiguous data.
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// # SAFETY
    ///
    /// - (i) `self` and `src` must have the same lengths.
    /// - (ii) `self` and `src` must not be overlapping.
    pub unsafe fn copy_from_nonoverlapping(&self, src: &Self) {
        debug_assert_eq!(self.len(), src.len());

        // SAFETY: (i) within bounds and (ii) slices do not overlap
        let dst = self.0 as *mut T;
        unsafe { dst.copy_from_nonoverlapping(src.0 as *const T, self.len()) };
    }
}
