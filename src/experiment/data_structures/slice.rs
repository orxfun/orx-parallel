use alloc::vec::Vec;
use core::{marker::PhantomData, ptr::slice_from_raw_parts};

/// A raw slice of contiguous data.
pub struct Slice<'a, T: 'a> {
    raw: *const [T],
    phantom: PhantomData<&'a ()>,
}

impl<'a, T: 'a> From<&[T]> for Slice<'a, T> {
    fn from(value: &[T]) -> Self {
        Self::new(value.as_ptr(), value.len())
    }
}

impl<'a, T: 'a> Slice<'a, T> {
    /// Creates a new raw slice.
    #[inline(always)]
    pub fn new(data: *const T, len: usize) -> Self {
        let raw = slice_from_raw_parts(data, len);
        let phantom = PhantomData;
        Self { raw, phantom }
    }

    pub fn empty() -> Self {
        // SAFETY: satisfies (i)
        unsafe { Self::new(core::ptr::null(), 0) }
    }

    pub fn from_vec_capacity(vec: &Vec<T>) -> Self {
        // SAFETY: constructing from a valid vec allocation.
        unsafe { Self::new(vec.as_ptr(), vec.capacity()) }
    }

    pub(super) fn data(&self) -> *const T {
        self.raw as *const T
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn last(&self) -> Option<*const T> {
        match self.len() {
            0 => None,
            n => Some(unsafe { self.data().add(n) }),
        }
    }

    /// # SAFETY
    ///
    /// - (i) `self` and `src` must have the same lengths.
    /// - (ii) `self` and `src` must not be overlapping.
    pub unsafe fn copy_from_nonoverlapping(&self, src: &Self) {
        debug_assert_eq!(self.len(), src.len());

        // SAFETY: (i) within bounds and (ii) slices do not overlap
        let dst = self.raw as *mut T;
        unsafe { dst.copy_from_nonoverlapping(src.raw as *const T, self.len()) };
    }
}

/// A struct holding a reference to a slice, hiding its unsafe methods,
/// allowing only safe methods.
pub struct SliceCore<'c, 'a, T: 'a>(&'c Slice<'a, T>);

impl<'c, 'a, T: 'a> SliceCore<'c, 'a, T> {
    pub fn new(slice: &'c Slice<'a, T>) -> Self {
        Self(slice)
    }
}

impl<'c, 'a, T: 'a> From<&'c Slice<'a, T>> for SliceCore<'c, 'a, T> {
    fn from(value: &'c Slice<'a, T>) -> Self {
        SliceCore::new(value)
    }
}

impl<'c, 'a, T: 'a> SliceCore<'c, 'a, T> {
    /// Returns true if slices `self` and `other` are non-overlapping.
    pub fn is_non_overlapping(&self, other: &Self) -> bool {
        match (self.0.len(), other.0.len()) {
            (0, _) | (_, 0) => true,
            (n, m) => {
                let diff = unsafe { self.0.data().offset_from(other.0.data()) };
                let (left, right) = match diff >= 0 {
                    true => (unsafe { other.0.data().add(m - 1) }, self.0.data()),
                    false => (unsafe { self.0.data().add(n - 1) }, other.0.data()),
                };
                left < right
            }
        }
    }
}
