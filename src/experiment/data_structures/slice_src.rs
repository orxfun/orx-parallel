use crate::experiment::data_structures::slice::Slice;

/// A raw slice of contiguous data with initialized values.
///
/// # SAFETY
///
/// While constructing this slice, we must guarantee that all elements of it
/// are initialized since it will be used as source of values.
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
}
