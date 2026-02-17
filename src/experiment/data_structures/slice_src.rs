use crate::experiment::data_structures::{
    slice::{Slice, SliceSafe},
    slice_iter_ptr_src::SliceIterPtrSrc,
};

/// A raw slice of contiguous data with initialized values.
///
/// # SAFETY
///
/// While constructing this slice, we must guarantee that all elements of it
/// are initialized since it will be used as source of values.
///
/// This is a read-only slice.
/// The caller must make sure that there is no concurrent write to this slice.
pub struct SliceSrc<'a, T>(Slice<'a, T>);

impl<'a, T> Clone for SliceSrc<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, T> SliceSrc<'a, T> {
    /// Destructs the slice wrapper and returns the underlying raw slice
    /// that it is created with.
    pub fn destruct(self) -> *const [T] {
        self.0.destruct()
    }

    /// Creates the source slice from the given `slice`.
    ///
    /// # SAFETY
    ///
    /// The `slice` guarantees that all elements are initialized.
    ///
    /// Further, this slice cannot outlive the `slice` it is created for due to the lifetime relation.
    pub fn from_slice(slice: &'a [T]) -> Self {
        Self(Slice::new(slice.as_ptr(), slice.len()))
    }

    /// Length of the slice.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a safe wrapper over this slice.
    #[inline(always)]
    pub fn core(&self) -> SliceSafe<'_, 'a, T> {
        self.into()
    }

    /// Returns a reference to the first element of the slice.
    ///
    /// # SAFETY
    ///
    /// - (i) this must have a positive `len`
    #[inline(always)]
    pub unsafe fn first_unchecked(&self) -> &'a T {
        // SAFETY: req't (i) is satisfied by cond'n (i);
        // req't (ii) is satisfied by SliceSrc construction.
        unsafe { self.0.first_unchecked() }
    }

    /// Converts the source slice into a source iterator.
    pub fn into_iter(self) -> SliceIterPtrSrc<'a, T> {
        SliceIterPtrSrc::new(self)
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
        // SAFETY: req't (i) is satisfied by cond'n (i)
        unsafe { self.0.split_at_unchecked(position) }.map(Self)
    }

    /// # SAFETY
    ///
    /// - (i) the values must not be mutated before the returned slice is dropped.
    pub unsafe fn as_slice(&self) -> &[T] {
        // SAFETY: (i) all values are initialized by construction,
        // (ii) is satisfied by cond'n (i).
        unsafe { self.0.as_slice() }
    }
}

impl<'c, 'a, T: 'a> From<&'c SliceSrc<'a, T>> for SliceSafe<'c, 'a, T> {
    fn from(value: &'c SliceSrc<'a, T>) -> Self {
        SliceSafe::new(&value.0)
    }
}
