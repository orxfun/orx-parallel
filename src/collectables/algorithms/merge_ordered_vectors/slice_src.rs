use super::slice::Slice;
use crate::results::ValIdx;

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
    pub fn destruct(self) -> *const [ValIdx<T>] {
        self.0.destruct()
    }

    pub fn from_slice(slice: &'a [ValIdx<T>]) -> Self {
        Self(Slice::new(slice.as_ptr(), slice.len()))
    }
}
