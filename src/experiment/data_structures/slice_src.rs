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

    /// Converts the source slice into a source iterator.
    pub fn into_iter(self) -> SliceIterPtrSrc<'a, T> {
        SliceIterPtrSrc::new(self)
    }
}

impl<'c, 'a, T: 'a> From<&'c SliceSrc<'a, T>> for SliceSafe<'c, 'a, T> {
    fn from(value: &'c SliceSrc<'a, T>) -> Self {
        SliceSafe::new(&value.0)
    }
}
