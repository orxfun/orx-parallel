use crate::experiment::data_structures::slice::Slice;

/// A raw slice of contiguous data with initialized values.
///
/// # SAFETY
///
/// While constructing this slice, we must guarantee that all elements of it
/// are initialized since it will be used as source of values.
pub struct SliceSrc<'a, T>(Slice<'a, T>);

impl<'a, T> From<&'a [T]> for SliceSrc<'a, T> {
    fn from(value: &'a [T]) -> Self {
        // # SAFETY: value initialization is guaranteed by the slice &[T]
        Self(Slice::new(value.as_ptr(), value.len()))
    }
}
