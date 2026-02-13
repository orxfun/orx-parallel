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

/// Extension methods for slices.
pub trait SliceExtensions<T> {
    fn empty() -> Self;

    fn from_vec_capacity(vec: &Vec<T>) -> Self;

    /// # SAFETY
    ///
    /// - (i) `begin` must be a non-null pointer within this slice.
    unsafe fn subslice_from_ptr(&self, begin: *const T) -> Self;

    /// # SAFETY
    ///
    /// - (i) `self` and `src` must have the same lengths.
    /// - (ii) `self` and `src` must not be overlapping.
    fn copy_from_nonoverlapping(&self, src: &Self);
}

impl<T> SliceExtensions<T> for *const [T] {
    fn empty() -> Self {
        slice_from_raw_parts(core::ptr::null(), 0)
    }

    fn from_vec_capacity(vec: &Vec<T>) -> Self {
        // SAFETY: constructing from a valid vec allocation.
        unsafe { new_slice(vec.as_ptr(), vec.capacity()) }
    }

    unsafe fn subslice_from_ptr(&self, begin: *const T) -> Self {
        let data = *self as *const T;

        debug_assert!(unsafe { begin.offset_from(data) >= 0 });
        debug_assert!(unsafe { data.add(self.len()).offset_from(begin) > 0 });

        // SAFETY: (i) `begin` is a non-null pointer within this slice.
        let begin_idx = unsafe { begin.offset_from(data) as usize };

        let len = self.len() - begin_idx;

        // SAFETY: (ii) `len` from `begin` is guaranteed to stay within bounds of this slice
        unsafe { new_slice(begin, len) }
    }

    fn copy_from_nonoverlapping(&self, src: &Self) {
        debug_assert_eq!(self.len(), src.len());
        let dst = *self as *mut T;

        //
        unsafe { dst.copy_from_nonoverlapping(*src as *const T, self.len()) };
    }
}

// fn copy_from_nonoverlapping(&self, source: &Self) {
//         debug_assert_eq!(self.len, source.len);
//         let dst = self.data as *mut T;
//         unsafe { dst.copy_from_nonoverlapping(source.data, source.len) };
//     }
