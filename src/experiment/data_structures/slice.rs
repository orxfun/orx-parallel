use core::{marker::PhantomData, ptr::slice_from_raw_parts, slice::from_raw_parts};

/// A raw slice of contiguous data.
pub struct Slice<'a, T: 'a> {
    raw: *const [T],
    phantom: PhantomData<&'a ()>,
}

impl<'a, T: 'a> Clone for Slice<'a, T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            phantom: PhantomData,
        }
    }
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

    /// Destructs the slice wrapper and returns the underlying raw slice
    /// that it is created with.
    pub fn destruct(self) -> *const [T] {
        self.raw
    }

    /// Creates two slices from this slice:
    ///
    /// - first slice for positions [0..position)
    /// - second slice for positions [position..]
    ///
    /// # SAFETY
    ///
    /// - (i) `position` must be less than or equal to `self.len()`
    pub unsafe fn split_at_unchecked(self, position: usize) -> [Self; 2] {
        let len_right = self.len() - position;
        let ptr_left = self.raw as *const T;
        let left = Self::new(ptr_left, position);
        // SAFETY: ptr_right is within bounds due to condition (i)
        let ptr_right = unsafe { ptr_left.add(position) };
        let right = Self::new(ptr_right, len_right);
        [left, right]
    }

    pub(super) fn data(&self) -> *const T {
        self.raw as *const T
    }

    /// Returns the length of the slice.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns true if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
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
pub struct SliceSafe<'c, 'a, T: 'a>(&'c Slice<'a, T>);

impl<'c, 'a, T: 'a> SliceSafe<'c, 'a, T> {
    /// Creates a safe wrapper over the `slice`.
    pub fn new(slice: &'c Slice<'a, T>) -> Self {
        Self(slice)
    }
}

impl<'c, 'a, T: 'a> From<&'c Slice<'a, T>> for SliceSafe<'c, 'a, T> {
    fn from(value: &'c Slice<'a, T>) -> Self {
        SliceSafe::new(value)
    }
}

impl<'c, 'a, T: 'a> SliceSafe<'c, 'a, T> {
    /// Returns true if slices `self` and `other` are non-overlapping.
    pub fn is_non_overlapping(&self, other: &Self) -> bool {
        match (self.0.len(), other.0.len()) {
            (0, _) | (_, 0) => true,
            (n, m) => {
                let (left, right) = match self.0.data() >= other.0.data() {
                    true => (unsafe { other.0.data().add(m - 1) }, self.0.data()),
                    false => (unsafe { self.0.data().add(n - 1) }, other.0.data()),
                };
                left < right
            }
        }
    }
}
