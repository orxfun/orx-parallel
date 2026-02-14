use crate::experiment::data_structures::slice::{Slice, SliceCore};

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

    #[inline(always)]
    pub fn core(&self) -> SliceCore<'_, 'a, T> {
        self.into()
    }
}

impl<'c, 'a, T: 'a> From<&'c SliceSrc<'a, T>> for SliceCore<'c, 'a, T> {
    fn from(value: &'c SliceSrc<'a, T>) -> Self {
        SliceCore::new(&value.0)
    }
}
