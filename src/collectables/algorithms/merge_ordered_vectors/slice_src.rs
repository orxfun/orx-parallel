use crate::results::ValIdx;
use core::{marker::PhantomData, ptr::slice_from_raw_parts};

/// A raw slice of contiguous data with initialized values.
///
/// # SAFETY
///
/// While constructing this slice, we must guarantee that all elements of it
/// are initialized since it will be used as source of values.
///
/// This is a read-only slice.
/// The caller must make sure that there is no concurrent write to this slice.
pub struct SliceSrc<'a, T> {
    raw: *const [ValIdx<T>],
    phantom: PhantomData<&'a ()>,
}

impl<'a, T> Clone for SliceSrc<'a, T> {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> SliceSrc<'a, T> {
    pub fn from_slice(slice: &'a [ValIdx<T>]) -> Self {
        let (data, len) = (slice.as_ptr(), slice.len());
        let raw = slice_from_raw_parts(data, len);
        let phantom = PhantomData;
        Self { raw, phantom }
    }

    pub fn destruct(self) -> *const [ValIdx<T>] {
        self.raw
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.raw.len()
    }
}
