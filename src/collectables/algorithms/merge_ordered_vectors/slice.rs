use crate::results::ValIdx;
use core::{marker::PhantomData, ptr::slice_from_raw_parts};

/// A raw slice of contiguous data.
pub struct Slice<'a, T: 'a> {
    raw: *const [ValIdx<T>],
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

impl<'a, T: 'a> From<&[ValIdx<T>]> for Slice<'a, T> {
    fn from(value: &[ValIdx<T>]) -> Self {
        Self::new(value.as_ptr(), value.len())
    }
}

impl<'a, T: 'a> Slice<'a, T> {
    #[inline(always)]
    pub fn new(data: *const ValIdx<T>, len: usize) -> Self {
        let raw = slice_from_raw_parts(data, len);
        let phantom = PhantomData;
        Self { raw, phantom }
    }

    pub fn destruct(self) -> *const [ValIdx<T>] {
        self.raw
    }
}
