use alloc::vec::Vec;
use core::ptr::slice_from_raw_parts_mut;

pub struct Slice<T> {
    pub data: *mut T,
    pub len: usize,
}
unsafe impl<T> Send for Slice<T> {}
unsafe impl<T> Sync for Slice<T> {}

impl<T> From<&mut [T]> for Slice<T> {
    fn from(value: &mut [T]) -> Self {
        Self {
            data: value.as_mut_ptr(),
            len: value.len(),
        }
    }
}

impl<T> Slice<T> {
    pub fn new(data: *mut T, len: usize) -> Self {
        Self { data, len }
    }

    pub fn slice_chunks(data: *mut T, len: usize, num_chunks: usize) -> Vec<Slice<T>> {
        let num_chunks = match num_chunks > len {
            true => len,
            false => num_chunks,
        };

        let mut slices = Vec::with_capacity(num_chunks);

        match num_chunks {
            0 => {}
            _ => {
                let avg_len = len / num_chunks;
                let lower_sum = avg_len * num_chunks;
                let num_larger = len - lower_sum;

                let mut begin = 0;
                let chunk_size = |c: usize| match c < num_larger {
                    true => avg_len + 1,
                    false => avg_len,
                };
                for c in 0..num_chunks {
                    let data = unsafe { data.add(begin) };
                    let len = chunk_size(c);
                    slices.push(Slice { data, len });

                    let end = begin + len;
                    begin = end;
                }
            }
        }

        slices
    }

    pub fn as_mut_slice(&self) -> &mut [T] {
        unsafe { &mut *slice_from_raw_parts_mut(self.data, self.len) }
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        match i < self.len {
            true => Some(unsafe { &*self.data.add(i) }),
            false => None,
        }
    }

    #[inline(always)]
    pub unsafe fn ptr_at(&self, i: usize) -> *const T {
        unsafe { &*self.data.add(i) }
    }

    /// Returns the slice obtained by merging all `slices`.
    ///
    /// # Panics
    ///
    /// if `slices` is empty.
    ///
    /// # Safety
    ///
    /// The `slices` are expected to be contiguous, which can create a large slice
    /// when joined back to back.
    pub fn merged_slice(slices: &[Slice<T>]) -> Self {
        debug_assert!(!slices.is_empty());

        let mut len = slices[0].len;
        let data = slices[0].data;
        let mut end = unsafe { data.add(slices[0].len) };

        for slice in slices.iter().skip(1) {
            debug_assert_eq!(slice.data, end);
            len += slice.len;
            end = unsafe { end.add(slice.len) };
        }
        Self { data, len }
    }
}
