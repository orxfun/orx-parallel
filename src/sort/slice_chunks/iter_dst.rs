use crate::sort::slice_chunks::iter_ptr::SliceIterPtr;

pub struct SliceIterDst<T> {
    data: *mut T,
    exclusive_end: *mut T,
}

impl<T> Default for SliceIterDst<T> {
    fn default() -> Self {
        Self {
            data: core::ptr::null_mut(),
            exclusive_end: core::ptr::null_mut(),
        }
    }
}

impl<T> SliceIterDst<T> {
    pub fn new(data: *mut T, len: usize) -> Self {
        match len {
            0 => Self::default(),
            _ => Self {
                data,
                exclusive_end: unsafe { data.add(len) },
            },
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        unsafe { self.exclusive_end.offset_from(self.data) as usize }
    }

    #[inline(always)]
    pub fn write_one(&mut self, src: *const T) {
        debug_assert!(self.len() > 0);
        unsafe { self.data.copy_from_nonoverlapping(src, 1) };
        self.data = unsafe { self.data.add(1) };
    }

    #[inline(always)]
    pub fn write_remaining_from(&mut self, iter: &SliceIterPtr<'_, T>) {
        debug_assert_eq!(self.len(), iter.len());
        if let Some(src) = iter.peek() {
            let len = iter.len();
            unsafe { self.data.copy_from_nonoverlapping(src, len) };
            self.data = unsafe { self.data.add(len) };
        }
        debug_assert_eq!(self.len(), 0);
    }
}
