use core::{marker::PhantomData, ops::Index};

/// A slice of contiguous data.
///
/// Its lifetime is bound to the owner of the data.
pub struct Slice<'a, T> {
    data: *const T,
    len: usize,
    phantom: PhantomData<&'a ()>,
}

/// SAFETY: [`Slice`] containing a raw pointer is allowed to send across threads.
/// Since the lifetime of the slice is bound to the owner of the data, this pointer
/// will be valid.
unsafe impl<T: Send> Send for Slice<'_, T> {}

/// SAFETY: [`Slice`] containing a raw pointer is allowed to shared across threads.
/// Since the lifetime of the slice is bound to the owner of the data, this pointer
/// will be valid.
unsafe impl<T: Sync> Sync for Slice<'_, T> {}

impl<'a, T> Slice<'a, T> {
    pub fn get(&self, index: usize) -> Option<&'a T> {
        match index < self.len {
            // # SAFETY: index is within the bounds and data is a valid pointer.
            true => Some(unsafe { &*self.data.add(index) }),
            false => None,
        }
    }
}
